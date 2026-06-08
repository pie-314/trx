"use client";

import React, { useEffect, useRef, useState, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";

export default function TrxDemoTerminal() {
  const terminalRef = useRef<HTMLDivElement>(null);
  const termInstance = useRef<Terminal | null>(null);
  const [isPaused, setIsPaused] = useState(false);
  const isPausedRef = useRef(false);
  const abortControllerRef = useRef<AbortController | null>(null);

  useEffect(() => {
    isPausedRef.current = isPaused;
  }, [isPaused]);

  const sleep = useCallback((ms: number, signal?: AbortSignal) => {
    return new Promise<void>((resolve, reject) => {
      const timeoutId = setTimeout(resolve, ms);
      if (signal) {
        signal.addEventListener("abort", () => {
          clearTimeout(timeoutId);
          reject(new DOMException("Aborted", "AbortError"));
        });
      }
    });
  }, []);

  const waitIfPaused = useCallback(async (signal?: AbortSignal) => {
    while (isPausedRef.current) {
      if (signal?.aborted) throw new DOMException("Aborted", "AbortError");
      await sleep(100, signal);
    }
  }, [sleep]);

  const typeCommand = useCallback(async (
    term: Terminal,
    command: string,
    signal: AbortSignal
  ) => {
    term.write("\r\n$ ");
    for (let i = 0; i < command.length; i++) {
      await waitIfPaused(signal);
      if (signal.aborted) return;
      term.write(command[i]);
      await sleep(50 + Math.random() * 100, signal);
    }
    await sleep(200, signal);
    term.write("\r\n");
  }, [sleep, waitIfPaused]);

  const runDemo = useCallback(async (signal: AbortSignal) => {
    if (!termInstance.current) return;
    const term = termInstance.current;

    try {
      while (!signal.aborted) {
        term.clear();
        await sleep(500, signal);
        await typeCommand(term, "trx neovim", signal);

        await waitIfPaused(signal);
        term.write("Searching... \x1b[32m████████████\x1b[0m 100%\r\n\r\n");
        await sleep(800, signal);
        await waitIfPaused(signal);

        term.write('Results for "neovim" (12 packages):\r\n\r\n');
        await sleep(300, signal);

        const results = [
          {
            name: "neovim",
            version: "0.9.5",
            desc: "Modern vim fork with Lua support",
          },
          { name: "nvim-qt", version: "0.5.0", desc: "Neovim GUI client" },
          {
            name: "neovim-git",
            version: "0.10.0",
            desc: "Neovim development build",
          },
        ];

        for (const res of results) {
          await waitIfPaused(signal);
          term.write(
            `  \x1b[32m✓\x1b[0m \x1b[36m${res.name.padEnd(
              15
            )}\x1b[0m ${res.version.padEnd(8)} \x1b[90m${res.desc}\x1b[0m\r\n`
          );
          await sleep(150, signal);
        }

        term.write("\r\n  ...\r\n");

        await sleep(3000, signal);
        await waitIfPaused(signal);
      }
    } catch (e) {
      if ((e as Error).name === "AbortError") {
        // Expected when stopped
      } else {
        console.error("Demo error:", e);
      }
    }
  }, [sleep, typeCommand, waitIfPaused]);

  const startDemo = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    abortControllerRef.current = new AbortController();
    runDemo(abortControllerRef.current.signal);
  }, [runDemo]);

  useEffect(() => {
    if (!terminalRef.current) return;

    const term = new Terminal({
      theme: {
        background: "#000000",
        foreground: "#ffffff",
        cursor: "#ffffff",
        selectionBackground: "#444444",
      },
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      fontSize: 14,
      cursorBlink: true,
      disableStdin: true,
      rows: 15,
    });

    termInstance.current = term;
    term.open(terminalRef.current);

    startDemo();

    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
      term.dispose();
      termInstance.current = null;
    };
  }, [startDemo]);

  const handlePause = () => setIsPaused((p) => !p);
  const handleReplay = () => {
    setIsPaused(false);
    startDemo();
  };

  return (
    <div className="w-full max-w-3xl mx-auto flex flex-col gap-6 my-16">
      {/* Premium Terminal Container */}
      <div className="relative rounded-xl overflow-hidden bg-[#0d0d0d] border border-gray-800/80 shadow-[0_0_50px_-12px_rgba(255,255,255,0.1)] ring-1 ring-white/5">
        
        {/* macOS-style Window Header */}
        <div className="flex items-center px-4 py-3 bg-[#161616] border-b border-gray-800/50">
          <div className="flex gap-2">
            <div className="w-3 h-3 rounded-full bg-[#ff5f56] shadow-sm"></div>
            <div className="w-3 h-3 rounded-full bg-[#ffbd2e] shadow-sm"></div>
            <div className="w-3 h-3 rounded-full bg-[#27c93f] shadow-sm"></div>
          </div>
          <div className="absolute left-1/2 -translate-x-1/2 flex items-center gap-2 text-xs text-gray-500 font-medium font-mono">
            <span>bash</span>
            <span className="text-gray-700">—</span>
            <span>80x15</span>
          </div>
        </div>

        {/* Terminal Canvas Container */}
        <div className="p-5 pl-6">
          <div ref={terminalRef} className="h-64 sm:h-72 w-full [&_.xterm-viewport]:!bg-transparent" />
        </div>
      </div>

      {/* Controls */}
      <div className="flex gap-4 justify-center items-center">
        <button
          onClick={handlePause}
          className="flex items-center gap-2 px-6 py-2.5 text-sm font-semibold text-gray-300 bg-white/5 hover:bg-white/10 border border-white/10 rounded-full transition-all duration-300 hover:shadow-[0_0_15px_-3px_rgba(255,255,255,0.1)] hover:text-white"
        >
          {isPaused ? (
            <>
              <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z" /></svg>
              Resume
            </>
          ) : (
            <>
              <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" /></svg>
              Pause
            </>
          )}
        </button>
        <button
          onClick={handleReplay}
          className="flex items-center gap-2 px-6 py-2.5 text-sm font-semibold text-black bg-white hover:bg-gray-200 rounded-full transition-all duration-300 shadow-[0_0_20px_-5px_rgba(255,255,255,0.4)]"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth="2.5"><path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
          Replay
        </button>
      </div>
    </div>
  );
}
