import { execSync } from 'child_process';

const ports = [5173, 5174, 5175];

for (const port of ports) {
  try {
    if (process.platform === 'win32') {
      const output = execSync(`netstat -ano | findstr :${port}`).toString();
      const lines = output.trim().split('\n');
      for (const line of lines) {
        const parts = line.trim().split(/\s+/);
        const state = parts[3];
        const pid = parts[parts.length - 1];
        if (
          (state === 'LISTENING' || !state) &&
          pid &&
          !isNaN(Number(pid)) &&
          Number(pid) !== process.pid &&
          Number(pid) !== 0
        ) {
          try {
            execSync(`taskkill /F /PID ${pid}`, { stdio: 'ignore' });
            console.log(`[Pagina Port Guard] Terminated lingering process PID ${pid} holding port ${port}`);
          } catch (_) {}
        }
      }
    } else {
      execSync(`lsof -ti :${port} | xargs kill -9 2>/dev/null || true`, { stdio: 'ignore' });
    }
  } catch (_) {
    // Port is already free
  }
}
