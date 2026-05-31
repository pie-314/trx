"use client";

import { useState, useRef, useEffect } from "react";
import { motion, useReducedMotion } from "motion/react";
import Link from "next/link";
import { SiApple, SiArchlinux, SiDebian, SiGithub } from "react-icons/si";
import { LuArrowRight, LuArrowUpRight } from "react-icons/lu";
import { DINO_PATH } from "./dino-path";

const INSTALL_CMD = "curl -fsSL https://trx.pidev.tech/install.sh | sh";
const EASE = [0.16, 1, 0.3, 1] as const;

function TrxDinoLogo({ size = 80 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={Math.round(size * 0.86)}
      viewBox="220 260 580 500"
      fill="currentColor"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <path d={DINO_PATH} fillRule="evenodd" />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.8" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="20 6 9 17 4 12" />
    </svg>
  );
}

function CopyIcon() {
  return (
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="9" y="9" width="13" height="13" rx="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </svg>
  );
}

export function Hero() {
  const [copied, setCopied] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reducedMotion = useReducedMotion();

  useEffect(() => () => { if (timer.current) clearTimeout(timer.current); }, []);

  const handleCopy = async () => {
    if (timer.current) clearTimeout(timer.current);
    try {
      await navigator.clipboard.writeText(INSTALL_CMD);
      setCopied(true);
      timer.current = setTimeout(() => setCopied(false), 1800);
    } catch {
      setCopied(false);
    }
  };

  return (
    <div
      style={{
        position: "relative",
        width: "100%",
        minHeight: "100svh",
        background: "transparent",
        overflow: "hidden",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      {/* GitHub icon - top right */}
      <motion.a
        href="https://github.com/pie-314/trx"
        target="_blank"
        rel="noopener noreferrer"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.4, delay: 0.5, ease: "easeOut" }}
        whileTap={{ scale: 0.95 }}
        style={{
          position: "absolute",
          top: 20,
          right: 24,
          zIndex: 10,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          width: 40,
          height: 40,
          borderRadius: "50%",
          background: "#1c1c1c",
          boxShadow: "0 0 0 1px rgba(255,255,255,0.1), inset 0 1px 0 0 rgba(255,255,255,0.06)",
          color: "#D4D4D4",
          transition: "background 0.15s ease, box-shadow 0.15s ease",
          textDecoration: "none",
        }}
        aria-label="GitHub repository"
      >
        <SiGithub size={18} />
      </motion.a>

      {/* Main content */}
      <div
        style={{
          position: "relative",
          zIndex: 1,
          width: "100%",
          maxWidth: 880,
          padding: "64px 24px 80px",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          textAlign: "center",
        }}
      >
        {/* Logo block */}
        <motion.div
          initial={{ opacity: 0, transform: reducedMotion ? "translateY(0)" : "translateY(-10px)" }}
          animate={{ opacity: 1, transform: "translateY(0)" }}
          transition={{ duration: 0.55, ease: EASE }}
          style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 14, marginBottom: 32 }}
        >
          <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 20 }}>
            <div
              style={{
                fontFamily: "var(--font-geist-sans)",
                fontSize: "clamp(60px, 10vw, 96px)",
                fontWeight: 800,
                letterSpacing: "-0.055em",
                lineHeight: 1,
                background: "linear-gradient(to bottom, #ffffff 20%, #a3a3a3 100%)",
                WebkitBackgroundClip: "text",
                WebkitTextFillColor: "transparent",
                filter: "drop-shadow(0 2px 6px rgba(0,0,0,0.5))",
                userSelect: "none",
              }}
            >
              trx
            </div>
            <div style={{ color: "#ffffff", filter: "drop-shadow(0 2px 8px rgba(0,0,0,0.5))" }}>
              <TrxDinoLogo size={90} />
            </div>
          </div>
          <div
            style={{
              display: "inline-flex",
              alignItems: "center",
              padding: "5px 14px",
              borderRadius: 999,
              background: "#252525",
              boxShadow: "0 0 0 1px rgba(255,255,255,0.07)",
              fontFamily: "var(--font-geist-sans)",
              fontSize: 12,
              fontWeight: 500,
              color: "#888",
              letterSpacing: "-0.01em",
            }}
          >
            Fast. Keyboard-driven. Cross-platform.
          </div>
        </motion.div>

        {/* Headline */}
        <motion.h1
          initial={{ opacity: 0, transform: reducedMotion ? "translateY(0)" : "translateY(16px)" }}
          animate={{ opacity: 1, transform: "translateY(0)" }}
          transition={{ duration: 0.55, ease: EASE, delay: 0.07 }}
          style={{
            fontFamily: "var(--font-geist-sans)",
            fontSize: "clamp(30px, 4.5vw, 50px)",
            fontWeight: 700,
            letterSpacing: "-0.04em",
            lineHeight: 1.2,
            margin: "0 0 44px",
            padding: 0,
          }}
        >
          <span style={{ color: "#F5F5F5", display: "block" }}>Search, install, manage.</span>
          <span style={{ color: "#F5F5F5", display: "block" }}>One TUI for all your packages.</span>
        </motion.h1>

        {/* Install command */}
        <motion.div
          initial={{ opacity: 0, transform: reducedMotion ? "translateY(0)" : "translateY(20px)" }}
          animate={{ opacity: 1, transform: "translateY(0)" }}
          transition={{ duration: 0.6, ease: EASE, delay: 0.14 }}
          style={{ width: "100%", maxWidth: 580, marginBottom: 52 }}
        >
          <div className="cmd-border-wrap">
            <div
              style={{
                display: "flex",
                alignItems: "center",
                flexWrap: "wrap",
                background: "#101010",
                borderRadius: 14,
                padding: "13px 13px 13px 22px",
                gap: 12,
                boxShadow: "0 2px 8px rgba(0,0,0,0.12)",
              }}
            >
              <code
                style={{
                  flex: 1,
                  minWidth: 0,
                  fontFamily: "var(--font-geist-mono)",
                  fontSize: "13.5px",
                  color: "#DCDCDC",
                  letterSpacing: "-0.01em",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                  userSelect: "all",
                }}
              >
                <span style={{ color: "#666", marginRight: 8 }}>$</span>
                {INSTALL_CMD}
              </code>
              <button
                onClick={handleCopy}
                title={copied ? "Copied!" : "Copy to clipboard"}
                style={{
                  flexShrink: 0,
                  display: "inline-flex",
                  alignItems: "center",
                  gap: 6,
                  padding: "7px 13px",
                  borderRadius: 8,
                  border: "1px solid rgba(255,255,255,0.07)",
                  background: copied ? "rgba(34, 70, 48, 0.9)" : "rgba(38, 38, 38, 0.95)",
                  color: copied ? "#5eea8a" : "#909090",
                  fontSize: 12,
                  fontWeight: 500,
                  fontFamily: "var(--font-geist-sans)",
                  letterSpacing: "-0.01em",
                  cursor: "pointer",
                  transition: "background 0.18s ease, color 0.18s ease",
                  whiteSpace: "nowrap",
                }}
              >
                {copied ? <CheckIcon /> : <CopyIcon />}
                {copied ? "Copied!" : "Copy"}
              </button>
            </div>
          </div>
        </motion.div>

        {/* Bottom cluster */}
        <motion.div
          initial={{ opacity: 0, transform: reducedMotion ? "translateY(0)" : "translateY(24px)" }}
          animate={{ opacity: 1, transform: "translateY(0)" }}
          transition={{ duration: 0.65, ease: EASE, delay: 0.22 }}
          style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 18 }}
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8,
              fontFamily: "var(--font-geist-sans)",
              fontSize: 13,
              fontWeight: 500,
              color: "#888",
              letterSpacing: "-0.01em",
            }}
          >
            <span>Fast</span>
            <span style={{ color: "#444", fontSize: 8 }}>●</span>
            <span>Keyboard-driven</span>
            <span style={{ color: "#444", fontSize: 8 }}>●</span>
            <span>Cross-platform</span>
          </div>

          <div
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 14,
              padding: "10px 18px",
              background: "#1c1c1c",
              borderRadius: 12,
              boxShadow: "0 0 0 1px rgba(255,255,255,0.07), inset 0 1px 0 0 rgba(255,255,255,0.05)",
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: 10, color: "#888" }}>
              <SiApple size={17} />
              <SiArchlinux size={17} />
              <SiDebian size={17} />
            </div>
            <div style={{ width: 1, height: 18, background: "rgba(255,255,255,0.12)", flexShrink: 0 }} />
            <span
              style={{
                fontFamily: "var(--font-geist-sans)",
                fontSize: 13,
                fontWeight: 500,
                color: "#a3a3a3",
                letterSpacing: "-0.01em",
              }}
            >
              Works everywhere you work
            </span>
          </div>

          <div style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: 16, flexWrap: "wrap" }}>
            <Link
              href="#install"
              className="primary-btn"
              style={{
                display: "inline-flex",
                alignItems: "center",
                padding: "10px 22px",
                borderRadius: 999,
                background: "linear-gradient(to bottom, #8B7FF7, #6B5EE0)",
                border: "1px solid rgba(107,94,224,0.5)",
                color: "#FFFFFF",
                fontSize: 14,
                fontWeight: 600,
                fontFamily: "var(--font-geist-sans)",
                textDecoration: "none",
                letterSpacing: "-0.01em",
                boxShadow: "inset 0 1px 0 0 rgba(255,255,255,0.2)",
                transition: "background 0.15s ease",
              }}
            >
              <span>Get started</span>
              <LuArrowRight className="arrow-icon" size={15} strokeWidth={2.5} style={{ marginLeft: 6, transition: "transform 0.15s ease" }} />
            </Link>
            <motion.div whileTap={{ scale: 0.97 }} transition={{ type: "spring", stiffness: 500, damping: 20 }}>
              <a
                href="https://pie-314.github.io/trx/"
                target="_blank"
                rel="noopener noreferrer"
                className="secondary-btn"
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  padding: "10px 22px",
                  borderRadius: 999,
                  background: "#FFFFFF",
                  border: "none",
                  color: "#555555",
                  fontSize: 14,
                  fontWeight: 500,
                  fontFamily: "var(--font-geist-sans)",
                  textDecoration: "none",
                  letterSpacing: "-0.01em",
                  boxShadow: "0 0 0 1px rgba(0,0,0,0.08), 0 1px 2px -1px rgba(0,0,0,0.06), 0 2px 4px 0 rgba(0,0,0,0.04)",
                  transition: "background 0.15s ease",
                }}
              >
                <span>Read docs</span>
                <LuArrowUpRight className="arrow-icon-diagonal" size={15} strokeWidth={2.5} style={{ marginLeft: 6, transition: "transform 0.15s ease" }} />
              </a>
            </motion.div>
          </div>
        </motion.div>
      </div>

      {/* Scroll chevron */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.6, delay: 1.1, ease: "easeOut" }}
        style={{
          position: "absolute",
          bottom: 28,
          left: "50%",
          transform: "translateX(-50%)",
          zIndex: 1,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 3,
        }}
      >
        <motion.svg
          width="20" height="20" viewBox="0 0 24 24" fill="none"
          stroke="rgba(255,255,255,0.25)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"
          animate={{ y: [0, 4, 0] }}
          transition={{ duration: 1.8, repeat: Infinity, ease: "easeInOut" }}
        >
          <polyline points="6 9 12 15 18 9" />
        </motion.svg>
      </motion.div>

      <style>{`
        @property --cmd-angle {
          syntax: '<angle>';
          inherits: false;
          initial-value: 0deg;
        }
        @keyframes cmd-border-spin { to { --cmd-angle: 360deg; } }
        .cmd-border-wrap {
          border-radius: 20px;
          padding: 6px;
          background: conic-gradient(
            from var(--cmd-angle),
            rgba(139,127,247,0.85) 0deg, transparent 55deg,
            transparent 125deg, rgba(255,255,255,0.25) 180deg,
            transparent 235deg, transparent 305deg,
            rgba(139,127,247,0.85) 360deg
          );
          animation: cmd-border-spin 4s linear infinite;
        }
        .primary-btn:hover .arrow-icon { transform: translateX(4px); }
        .secondary-btn:hover .arrow-icon-diagonal { transform: translate(3px, -3px); }
        @media (prefers-reduced-motion: reduce) {
          .cmd-border-wrap {
            animation: none;
            background: conic-gradient(from 45deg, rgba(139,127,247,0.6), transparent, rgba(255,255,255,0.15), transparent, rgba(139,127,247,0.6));
          }
          .primary-btn:hover .arrow-icon,
          .secondary-btn:hover .arrow-icon-diagonal { transform: none !important; }
        }
      `}</style>
    </div>
  );
}
