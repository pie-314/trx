"use client";

import dynamic from "next/dynamic";

const TrxDemoTerminal = dynamic(() => import("./TrxDemoTerminal"), {
  ssr: false,
});

export default function TrxDemoWrapper() {
  return <TrxDemoTerminal />;
}
