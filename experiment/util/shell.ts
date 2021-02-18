import { exec } from 'child_process';

export async function execShellCommand(cmd: string): Promise<String> {
  return new Promise<String>((resolve, reject) => {
    exec(cmd, (error, stdout, stderr) => {
      if (error) {
        console.warn(error);
      }
      resolve(stdout ? stdout : stderr);
    });
  });
}