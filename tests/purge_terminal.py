#!/usr/bin/env python3
"""macOS PTY regression tests. All authentication and purge commands are mocked.

Run after cargo build: python3 tests/purge_terminal.py [--binary target/debug/clergy]
No real password, sudo authentication, DNS flush or snapshot thinning is performed.
"""
import argparse
import codecs
import fcntl
import json
import os
from pathlib import Path
import pty
import re
import select
import shutil
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/clergy")
parser.add_argument("--test", help="Run one named test method")
ARGS = parser.parse_args()
BINARY = ARGS.binary.resolve()

# Only this fixture consumes the dummy test password. Never log its value.
MOCK = r'''#!/usr/bin/python3
import json, os, signal, subprocess, sys, termios, time
name = os.path.basename(sys.argv[0])
args = sys.argv[1:]
mode = os.environ['TEST_MODE']
def log(event, **data):
    with open(os.environ['TEST_LOG'], 'a') as file:
        file.write(json.dumps(dict(event=event, **data)) + '\n')
if name == 'sudo':
    log('sudo', args=args)
    if args == ['-v']:
        with open('/dev/tty', 'r+b', buffering=0) as tty:
            fd = tty.fileno()
            saved = termios.tcgetattr(fd)
            log('auth', canonical=bool(saved[3] & termios.ICANON),
                echo=bool(saved[3] & termios.ECHO))
            hidden = termios.tcgetattr(fd)
            hidden[3] &= ~termios.ECHO
            termios.tcsetattr(fd, termios.TCSANOW, hidden)
            try:
                tty.write(b'Password:')
                os.read(fd, 4096)
            except KeyboardInterrupt:
                log('cancelled')
                sys.exit(1)
            finally:
                termios.tcsetattr(fd, termios.TCSANOW, saved)
                tty.write(b'\n')
        sys.exit(1 if mode == 'auth_failure' else 0)
    assert args[0] == '-n', 'Backend attempted interactive sudo!'
    if args[1:] == ['-v']:
        if mode == 'expired':
            print('sudo: a password is required', file=sys.stderr)
            sys.exit(1)
        sys.exit(0)
    sys.exit(subprocess.call(args[1:]))
elif name == 'tmutil' and args[0] == 'listlocalsnapshots':
    log('snapshot_inventory')
    if mode == 'inventory_failure':
        sys.exit(1)
    print('Snapshots for volume group containing disk /:')
    if not os.path.exists(os.environ['TEST_LOG'] + '.purged'):
        print('com.apple.TimeMachine.2026-10-04-190000.local')
    print('com.apple.TimeMachine.2026-10-04-191000.local')
elif name in ('dscacheutil', 'killall', 'tmutil'):
    log('action', name=name, args=args)
    if mode == 'backend_failure' and name == 'dscacheutil':
        print('mock DNS flush failed', file=sys.stderr)
        sys.exit(1)
    if name == 'tmutil':
        time.sleep(1.5)  # Prove the UI continues redrawing during slow work.
        if mode == 'snapshot_failure':
            print('mock snapshot thinning failed', file=sys.stderr)
            sys.exit(1)
        open(os.environ['TEST_LOG'] + '.purged', 'w').close()
        print('Thinned local snapshots:')
        print('com.apple.TimeMachine.2026-10-04-190000.local')
elif name == 'df':
    if '-kP' in args:
        if mode == 'disk_unavailable':
            sys.exit(1)
        available = 600002048 if os.path.exists(os.environ['TEST_LOG'] + '.purged') else 600000000
        print('Filesystem 1024-blocks Used Available Capacity Mounted on')
        print('/dev/disk1 976000000 10000000 %s 1%% /' % available)
    else:
        print('Filesystem Size Used Avail Capacity Mounted on')
        print('/dev/disk1 931Gi 13Gi 572Gi 1% /')
elif name == 'pagesize':
    print(os.environ.get('TEST_PAGE_SIZE', '16384'))
elif name == 'vm_stat':
    if mode == 'memory_unavailable':
        sys.exit(1)
    print('Pages free: 100.\nPages active: 200.\nPages inactive: 50.\n'
          'Pages speculative: 10.\nPages occupied by compressor: 20.')
elif name == 'sysctl':
    if mode == 'swap_unavailable':
        sys.exit(1)
    print('vm.swapusage: total = 1024.00M  used = 512.00M  free = 512.00M')
elif name == 'jq':
    if mode == 'invalid_json':
        print('invalid json')
    else:
        os.execv(os.environ['TEST_JQ'], [os.environ['TEST_JQ']] + args)
else:
    raise AssertionError(name)
'''

# Keep the controlling session alive long enough to inspect terminal restoration.
# macOS invalidates slave ioctls once the session leader exits.
RUNNER = r'''
import json, os, signal, subprocess, sys, termios
signal.signal(signal.SIGINT, lambda *_: None)
code = subprocess.call(sys.argv[1:])
with open(os.environ['TEST_LOG'], 'a') as file:
    file.write(json.dumps(dict(event='exit', code=code,
                               attributes=repr(termios.tcgetattr(0)))) + '\n')
sys.exit(code)
'''


class Screen:
    """Minimal CSI screen model for Ratatui's cursor-addressed output."""
    def __init__(self, rows=50, columns=140):
        self.rows, self.columns = rows, columns
        self.cells = [[" "] * columns for _ in range(rows)]
        self.row = self.column = 0
        self.pending = ""
        self.decoder = codecs.getincrementaldecoder("utf-8")("replace")

    def feed(self, data):
        self.pending += self.decoder.decode(data)
        while self.pending:
            if self.pending.startswith("\x1b["):
                match = re.match(r"\x1b\[([0-?]*)([ -/]*)([@-~])", self.pending)
                if not match:
                    break
                self.pending = self.pending[match.end():]
                parameters, _, command = match.groups()
                values = [int(value) if value else 0 for value in parameters.lstrip("?").split(";")]
                amount = values[0] or 1
                if command in ("H", "f"):
                    self.row = min(self.rows - 1, amount - 1)
                    self.column = min(self.columns - 1, (values[1] or 1) - 1 if len(values) > 1 else 0)
                elif command == "A":
                    self.row = max(0, self.row - amount)
                elif command == "B":
                    self.row = min(self.rows - 1, self.row + amount)
                elif command == "C":
                    self.column = min(self.columns - 1, self.column + amount)
                elif command == "D":
                    self.column = max(0, self.column - amount)
                elif command == "G":
                    self.column = min(self.columns - 1, amount - 1)
                elif command == "J" and values[0] in (2, 3):
                    self.cells = [[" "] * self.columns for _ in range(self.rows)]
                elif command == "K":
                    start = 0 if values[0] in (1, 2) else self.column
                    stop = self.column + 1 if values[0] == 1 else self.columns
                    self.cells[self.row][start:stop] = [" "] * (stop - start)
                continue
            if self.pending[0] == "\x1b":
                if len(self.pending) < 2:
                    break
                self.pending = self.pending[2:]
                continue
            char, self.pending = self.pending[0], self.pending[1:]
            if char == "\r":
                self.column = 0
            elif char == "\n":
                self.row = min(self.rows - 1, self.row + 1)
            elif char == "\b":
                self.column = max(0, self.column - 1)
            elif char >= " ":
                if self.column == self.columns:
                    self.column = 0
                    self.row = min(self.rows - 1, self.row + 1)
                self.cells[self.row][self.column] = char
                self.column += 1

    def text(self):
        return "\n".join("".join(row) for row in self.cells).encode()


class PurgeTerminalTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="clergy-pty-")
        self.base = Path(self.tmp.name)
        self.bin = self.base / "bin"
        self.bin.mkdir()
        fixture = self.bin / "mock.py"
        fixture.write_text(MOCK)
        fixture.chmod(0o755)
        for name in ("sudo", "dscacheutil", "killall", "tmutil", "pagesize", "df",
                     "vm_stat", "sysctl", "jq"):
            (self.bin / name).symlink_to(fixture)
        # An isolated PATH also lets us simulate missing jq without hiding mocks.
        for name in ("zsh", "awk", "date", "hostname", "whoami"):
            (self.bin / name).symlink_to(shutil.which(name))
        self.log = self.base / "calls.jsonl"
        self.runner = self.base / "runner.py"
        self.runner.write_text(RUNNER)
        self.env = dict(os.environ, PATH=str(self.bin), HOME=str(self.base),
                        TERM="xterm-256color", TEST_LOG=str(self.log),
                        TEST_MODE="success", TEST_JQ=shutil.which("jq"))
        self.process = None
        self.master = self.slave = None
        self.output = bytearray()
        self.screen = Screen()
        self.screen_history = bytearray()

    def tearDown(self):
        if self.process is not None and self.process.poll() is None:
            os.killpg(self.process.pid, signal.SIGKILL)
            self.process.wait(timeout=5)
        for fd in (self.master, self.slave):
            if fd is not None:
                os.close(fd)
        self.tmp.cleanup()

    def calls(self, event=None):
        calls = [json.loads(line) for line in self.log.read_text().splitlines()] if self.log.exists() else []
        return [call for call in calls if event is None or call["event"] == event]

    def pump(self, duration=0.15):
        end = time.monotonic() + duration
        while time.monotonic() < end:
            ready, _, _ = select.select([self.master], [], [], max(0, min(0.05, end - time.monotonic())))
            if ready:
                data = os.read(self.master, 65536)
                self.output.extend(data)
                self.screen.feed(data)
                self.screen_history.extend(self.screen.text())

    def wait_for(self, marker, timeout=10):
        def contains_text():
            plain = re.sub(rb"\x1b\[[0-?]*[ -/]*[@-~]", b"", bytes(self.output))
            # Ratatui may advance the cursor over spaces rather than write them.
            return re.sub(rb"\s+", b"", marker) in re.sub(rb"\s+", b"", plain + self.screen_history)
        end = time.monotonic() + timeout
        while not contains_text() and time.monotonic() < end:
            self.pump()
        if not contains_text():
            plain = re.sub(rb"\x1b\[[0-?]*[ -/]*[@-~]", b"", bytes(self.output))
            words = re.findall(rb"[A-Za-z][A-Za-z .:'()/-]{4,}", plain)
            self.fail("Expected %r; text: %r; calls: %r" %
                      (marker, words, self.calls()))

    def launch(self, mode="success", args=()):
        self.env["TEST_MODE"] = mode
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 50, 140, 0, 0))
        self.original = termios.tcgetattr(self.slave)

        def setup_tty():
            os.setsid()
            fcntl.ioctl(self.slave, termios.TIOCSCTTY, 0)

        self.process = subprocess.Popen(["/usr/bin/python3", str(self.runner), str(BINARY), *args], stdin=self.slave,
                                        stdout=self.slave, stderr=self.slave,
                                        env=self.env, preexec_fn=setup_tty)
        if not args:
            self.wait_for(b"Menu")
            self.assertEqual(self.calls("sudo"), [], "Dashboard must not authenticate")

    def confirm(self):
        os.write(self.master, b"\r")
        self.wait_for(b"Confirm Purge")
        os.write(self.master, b"\r")
        self.wait_for(b"Password:")
        auth = self.calls("auth")[-1]
        self.assertTrue(auth["canonical"], "sudo was started in raw mode")
        self.assertTrue(auth["echo"], "Original terminal mode was not restored")
        self.assertFalse(termios.tcgetattr(self.slave)[3] & termios.ECHO)
        before_prompt = bytes(self.output).split(b"Password:")[0]
        self.assertGreater(before_prompt.rfind(b"\x1b[?1049l"),
                           before_prompt.rfind(b"\x1b[?1049h"),
                           "Password prompt appeared inside the alternate screen")

    def password(self):
        os.write(self.master, b"fake-password-for-test\n")

    def quit_and_check(self):
        os.write(self.master, b"q")
        end = time.monotonic() + 10
        while self.process.poll() is None and time.monotonic() < end:
            self.pump()
        self.process.wait(timeout=1)
        self.pump()
        self.assertEqual(self.process.returncode, 0)
        self.assertEqual(self.calls("exit")[-1]["attributes"], repr(self.original))
        self.assertIn(b"\x1b[?25h", self.output)  # Cursor restored.
        self.assertNotIn(b"fake-password-for-test", self.output)
        self.assertGreater(self.output.rfind(b"\x1b[?1049l"),
                           self.output.rfind(b"\x1b[?1049h"))

    def test_dashboard_and_cancel_do_not_authenticate(self):
        self.launch()
        os.write(self.master, b"\r")
        self.wait_for(b"Confirm Purge")
        os.write(self.master, b"\t\r")  # Select Cancel.
        self.pump(0.5)
        self.assertEqual(self.calls(), [])
        self.quit_and_check()

    def test_success_keeps_ui_alive_and_blocks_duplicate_purges(self):
        self.launch()
        self.confirm()
        self.password()
        self.wait_for(b"Running system purge")
        self.pump(0.2)
        previous_length = len(self.output)
        os.write(self.master, b"\r\x1b[B\r\x1bq")
        self.pump(0.4)
        self.assertGreater(len(self.output), previous_length, "UI stopped redrawing")
        self.assertIsNone(self.process.poll(), "Running operation was orphaned")
        self.wait_for(b"Result")
        self.assertEqual(len(self.calls("auth")), 1)
        self.assertEqual([call["name"] for call in self.calls("action")],
                         ["dscacheutil", "killall", "tmutil"])
        os.write(self.master, b"\x1b")
        self.pump(0.3)
        os.write(self.master, b"\r\r")
        self.wait_for(b"Purge recently performed")
        self.assertEqual(len(self.calls("auth")), 1, "Cooldown must precede authentication")
        self.quit_and_check()

    def check_failure(self, mode, expected, actions):
        self.launch(mode)
        self.confirm()
        self.password()
        self.wait_for(expected)
        self.assertEqual(len(self.calls("action")), actions)
        self.quit_and_check()

    def test_authentication_failure_never_starts_backend(self):
        self.check_failure("auth_failure", b"authentication failed", 0)
        self.assertEqual(len(self.calls("sudo")), 1)

    def test_ctrl_c_cancels_authentication_and_restores_ui(self):
        self.launch()
        self.confirm()
        os.write(self.master, b"\x03")
        self.wait_for(b"authentication failed")
        self.assertEqual(self.calls("action"), [])
        self.quit_and_check()

    def test_expired_credentials_fail_without_second_prompt(self):
        self.check_failure("expired", b"credentials are unavailable", 0)
        self.assertEqual(len(self.calls("auth")), 1)

    def test_backend_failure_returns_to_ui(self):
        self.check_failure("backend_failure", b"mock DNS flush failed", 1)

    def test_invalid_json_returns_to_ui(self):
        self.check_failure("invalid_json", b"failed to parse backend JSON", 3)

    def test_missing_encoder_fails_before_system_changes(self):
        (self.bin / "jq").unlink()
        self.check_failure("success", b"Required command not found: jq", 0)

    def test_cli_password_prompt_preserves_json_stdout(self):
        # A terminal for stdin, a pipe for JSON output: matches clergy purge --json.
        self.master, self.slave = pty.openpty()
        original = termios.tcgetattr(self.slave)
        def setup_tty():
            os.setsid()
            fcntl.ioctl(self.slave, termios.TIOCSCTTY, 0)
        self.process = subprocess.Popen(["/usr/bin/python3", str(self.runner), str(BINARY), "purge", "--json"],
                                        stdin=self.slave, stdout=subprocess.PIPE,
                                        stderr=self.slave, env=self.env,
                                        preexec_fn=setup_tty)
        self.wait_for(b"Password:")
        self.password()
        output = bytearray()
        end = time.monotonic() + 10
        while self.process.poll() is None and time.monotonic() < end:
            self.pump(0.05)
            ready, _, _ = select.select([self.process.stdout], [], [], 0)
            if ready:
                output.extend(os.read(self.process.stdout.fileno(), 65536))
        self.process.wait(timeout=1)
        output.extend(self.process.stdout.read())
        self.process.stdout.close()
        self.pump()
        self.assertEqual(self.process.returncode, 0)
        self.assertTrue(json.loads(output)["snapshots_thinned"])
        report = json.loads(output)
        self.assertEqual([action["status"] for action in report["actions"]], ["succeeded"] * 3)
        self.assertTrue(all(action["exit_code"] == 0 for action in report["actions"]))
        self.assertEqual(report["disk_available_after_kib"] - report["disk_available_before_kib"], 2048)
        self.assertEqual(len(report["snapshots_before"]), 2)
        self.assertEqual(len(report["snapshots_after"]), 1)
        self.assertEqual(self.calls("exit")[-1]["attributes"], repr(original))
        self.assertNotIn(b"fake-password-for-test", self.output)

    def test_headless_noninteractive_arm_and_intel_page_sizes(self):
        for page_size in (16384, 4096):
            with self.subTest(page_size=page_size):
                self.env["TEST_PAGE_SIZE"] = str(page_size)
                result = subprocess.run([str(BINARY), "purge", "--json"],
                                        stdin=subprocess.DEVNULL, capture_output=True,
                                        env=self.env, timeout=10)
                self.assertEqual(result.returncode, 0, result.stderr.decode())
                data = json.loads(result.stdout)
                self.assertEqual(data["memory_before"]["used_mb"], 260 * page_size // 1048576)
                self.assertEqual(data["memory_before"]["compressed_mb"], 20 * page_size // 1048576)
                self.assertEqual(data["swap_before"]["used_mb"], 512)
                self.assertEqual(self.calls("auth"), [])
                self.assertTrue(all(call["args"][0] == "-n" for call in self.calls("sudo")))

    def test_partial_failure_retains_journal_and_returns_nonzero(self):
        self.env["TEST_MODE"] = "snapshot_failure"
        result = subprocess.run([str(BINARY), "purge", "--json"], stdin=subprocess.DEVNULL,
                                capture_output=True, env=self.env, timeout=10)
        self.assertNotEqual(result.returncode, 0)
        data = json.loads(result.stdout)
        self.assertTrue(data["dns_flushed"])
        self.assertFalse(data["snapshots_thinned"])
        self.assertEqual(data["actions"][-1]["status"], "failed")
        self.assertEqual(data["actions"][-1]["exit_code"], 1)
        self.assertIn("mock snapshot thinning failed", data["actions"][-1]["output"])

    def test_early_failure_records_skipped_steps(self):
        self.env["TEST_MODE"] = "backend_failure"
        result = subprocess.run([str(BINARY), "purge", "--json"], stdin=subprocess.DEVNULL,
                                capture_output=True, env=self.env, timeout=10)
        self.assertNotEqual(result.returncode, 0)
        data = json.loads(result.stdout)
        self.assertEqual([action["status"] for action in data["actions"]], ["failed", "skipped", "skipped"])
        self.assertEqual(len(self.calls("action")), 1)
        self.assertIsNone(data["actions"][1]["exit_code"])

    def test_snapshot_inventory_failure_is_unknown(self):
        self.env["TEST_MODE"] = "inventory_failure"
        result = subprocess.run([str(BINARY), "purge", "--json"], stdin=subprocess.DEVNULL,
                                capture_output=True, env=self.env, timeout=10)
        self.assertEqual(result.returncode, 0)
        data = json.loads(result.stdout)
        self.assertIsNone(data["snapshots_before"])
        self.assertIsNone(data["snapshots_after"])

    def test_unavailable_precise_disk_samples_do_not_become_zero(self):
        self.env["TEST_MODE"] = "disk_unavailable"
        result = subprocess.run([str(BINARY), "purge", "--json"], stdin=subprocess.DEVNULL,
                                capture_output=True, env=self.env, timeout=10)
        self.assertEqual(result.returncode, 0)
        data = json.loads(result.stdout)
        self.assertIsNone(data["disk_available_before_kib"])
        self.assertIsNone(data["disk_available_after_kib"])

    def test_report_scrolls_and_last_session_reopens_without_authentication(self):
        self.launch()
        self.confirm()
        self.password()
        self.wait_for(b"PURGE COMPLETE")
        self.wait_for(b"WHAT HAPPENED")
        self.wait_for(b"MEASURED CHANGES")
        self.assertIn(b"DNS cache refreshed", self.screen.text())
        self.assertNotIn(b"sudo -n", self.screen.text())
        self.assertNotIn(b"exit 0", self.screen.text())
        os.write(self.master, b"\x1b[F")  # End.
        self.wait_for(b"binary units")
        os.write(self.master, b"\x1b[H")  # Home.
        self.pump(0.3)
        self.assertIn(b"PURGE COMPLETE", self.screen.text())
        os.write(self.master, b"\x1b")
        self.pump(0.3)
        os.write(self.master, b"\x1b[B\r")  # Last Session.
        self.pump(0.5)
        self.assertIn(b"PURGE COMPLETE", self.screen.text())
        self.assertEqual(len(self.calls("auth")), 1)
        self.quit_and_check()

    def test_unavailable_memory_and_swap_samples_are_flagged(self):
        for mode, metric in [("memory_unavailable", "memory"), ("swap_unavailable", "swap")]:
            with self.subTest(mode=mode):
                self.env["TEST_MODE"] = mode
                result = subprocess.run([str(BINARY), "purge", "--json"], stdin=subprocess.DEVNULL,
                                        capture_output=True, env=self.env, timeout=10)
                self.assertEqual(result.returncode, 0)
                data = json.loads(result.stdout)
                self.assertFalse(data[metric + "_before_available"])
                self.assertFalse(data[metric + "_after_available"])


if __name__ == "__main__":
    if sys.platform != "darwin":
        sys.exit("These PTY/backend regression tests require macOS.")
    if not BINARY.is_file() or not shutil.which("jq"):
        sys.exit("Build clergy first and install jq (brew install jq).")
    unittest.main(argv=[sys.argv[0]] + (["PurgeTerminalTests." + ARGS.test] if ARGS.test else []),
                  verbosity=2)
