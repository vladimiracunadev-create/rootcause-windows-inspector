import * as vscode from 'vscode';
import { execFile } from 'child_process';
import { promisify } from 'util';

const execFileAsync = promisify(execFile);

interface RootCauseStatus {
    severity: string;
    cpu_percent: number;
    ram_percent: number;
    io_mbps?: number;
}

function getExecutable(): string {
    const cfg = vscode.workspace.getConfiguration('rootcause');
    return cfg.get<string>('executablePath', 'rootcause');
}

function getInterval(): number {
    const cfg = vscode.workspace.getConfiguration('rootcause');
    return cfg.get<number>('refreshIntervalSeconds', 15) * 1000;
}

function severityIcon(severity: string): string {
    switch (severity.toLowerCase()) {
        case 'critical': return '$(error)';
        case 'high':     return '$(warning)';
        case 'medium':   return '$(info)';
        default:         return '$(check)';
    }
}

async function fetchStatus(exe: string): Promise<RootCauseStatus | null> {
    try {
        const { stdout } = await execFileAsync(exe, ['status', '--json'], { timeout: 8000 });
        return JSON.parse(stdout.trim()) as RootCauseStatus;
    } catch {
        return null;
    }
}

export function activate(context: vscode.ExtensionContext): void {
    const cfg = vscode.workspace.getConfiguration('rootcause');
    const showBar = cfg.get<boolean>('showStatusBar', true);

    const bar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    bar.tooltip = 'RootCause Windows Inspector — haz clic para abrir el panel';
    bar.command = 'rootcause.openPanel';
    if (showBar) bar.show();
    context.subscriptions.push(bar);

    let lastSeverity = '';
    let pollTimer: ReturnType<typeof setTimeout> | undefined;

    async function refresh(): Promise<void> {
        const exe = getExecutable();
        const status = await fetchStatus(exe);

        if (!status) {
            bar.text = '$(error) RootCause: sin datos';
            bar.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
            return;
        }

        const icon = severityIcon(status.severity);
        const io = status.io_mbps != null ? ` I/O:${status.io_mbps.toFixed(1)}` : '';
        bar.text = `${icon} CPU:${status.cpu_percent.toFixed(0)}% RAM:${status.ram_percent.toFixed(0)}%${io}`;

        const isCritical = status.severity.toLowerCase() === 'critical';
        bar.backgroundColor = isCritical
            ? new vscode.ThemeColor('statusBarItem.errorBackground')
            : undefined;

        const alertOn = vscode.workspace.getConfiguration('rootcause').get<boolean>('alertOnCritical', true);
        if (alertOn && isCritical && lastSeverity !== 'critical') {
            vscode.window.showWarningMessage(
                `RootCause: estado CRÍTICO — CPU ${status.cpu_percent.toFixed(0)}% RAM ${status.ram_percent.toFixed(0)}%`,
                'Abrir panel'
            ).then(choice => {
                if (choice === 'Abrir panel') {
                    vscode.commands.executeCommand('rootcause.openPanel');
                }
            });
        }
        lastSeverity = status.severity.toLowerCase();
    }

    function schedulePoll(): void {
        pollTimer = setTimeout(async () => {
            await refresh();
            schedulePoll();
        }, getInterval());
    }

    // First paint immediately
    refresh().then(() => schedulePoll());

    // Commands
    context.subscriptions.push(
        vscode.commands.registerCommand('rootcause.refresh', async () => {
            if (pollTimer) clearTimeout(pollTimer);
            await refresh();
            schedulePoll();
            vscode.window.setStatusBarMessage('RootCause: actualizado', 3000);
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('rootcause.export', async () => {
            const exe = getExecutable();
            try {
                const { stdout } = await execFileAsync(exe, ['export'], { timeout: 15000 });
                const line = stdout.trim().split('\n')[0];
                vscode.window.showInformationMessage(`RootCause: ${line}`);
            } catch (err) {
                vscode.window.showErrorMessage(`RootCause export falló: ${(err as Error).message}`);
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('rootcause.openPanel', async () => {
            const panel = vscode.window.createWebviewPanel(
                'rootcausePanel',
                'RootCause — Diagnóstico',
                vscode.ViewColumn.Two,
                { enableScripts: false }
            );

            const exe = getExecutable();
            let snapshotText = 'Cargando…';
            try {
                const { stdout } = await execFileAsync(exe, ['snapshot'], { timeout: 10000 });
                snapshotText = escapeHtml(stdout.trim());
            } catch (err) {
                snapshotText = `Error: ${escapeHtml((err as Error).message)}`;
            }

            panel.webview.html = buildPanelHtml(snapshotText);
        })
    );

    context.subscriptions.push({ dispose: () => { if (pollTimer) clearTimeout(pollTimer); } });
}

export function deactivate(): void { /* nothing to tear down */ }

function escapeHtml(s: string): string {
    return s
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
}

function buildPanelHtml(content: string): string {
    return `<!DOCTYPE html>
<html lang="es">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>RootCause</title>
  <style>
    body { font-family: var(--vscode-editor-font-family, monospace); font-size: 13px;
           background: var(--vscode-editor-background); color: var(--vscode-editor-foreground);
           padding: 16px; }
    pre  { white-space: pre-wrap; word-break: break-word; }
    h2   { color: var(--vscode-titleBar-activeForeground, #ccc); margin-bottom: 8px; }
  </style>
</head>
<body>
  <h2>RootCause Windows Inspector — snapshot</h2>
  <pre>${content}</pre>
</body>
</html>`;
}
