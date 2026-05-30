"use client";

import React from "react";
import { DocsLayout } from "@/components/DocsLayout";
import { CodeBlock } from "@/components/CodeBlock";

export default function NonNativeMathPage() {
  return (
    <DocsLayout>
      <div className="mb-10">
        <div className="flex items-center gap-2 mb-3">
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-cyan-100 dark:bg-cyan-900/30 text-cyan-700 dark:text-cyan-300 border border-cyan-200 dark:border-cyan-800 uppercase tracking-wider">
            API Reference
          </span>
        </div>
        <h1 className="text-4xl md:text-5xl font-extrabold text-black dark:text-white tracking-tight mb-4">
          Non-Native Math Integration Guide
        </h1>
        <p className="text-lg text-neutral-500 dark:text-neutral-400 leading-relaxed max-w-3xl">
          How to use multi-limb foreign field wrappers in Soroban-ZK-Std without
          losing canonical form, range safety, or host compatibility.
        </p>
      </div>

      <hr className="border-neutral-200 dark:border-neutral-800 mb-10" />

      <section className="mb-12">
        <h2 className="text-2xl font-bold text-black dark:text-white tracking-tight mb-4">
          Overview
        </h2>
        <p className="text-neutral-600 dark:text-neutral-400 leading-relaxed mb-4">
          Non-native math appears when a value is represented in a field that is
          not the one used by the current arithmetic engine. In this library,
          that usually means one of two boundaries:
        </p>
        <ul className="space-y-3 text-neutral-600 dark:text-neutral-400 ml-4">
          <li className="flex gap-3">
            <span className="text-black dark:text-white font-bold shrink-0">1.</span>
            <span>
              A Soroban-managed <code className="px-1.5 py-0.5 bg-neutral-100 dark:bg-neutral-800 rounded text-sm font-mono text-black dark:text-white">U256</code>{" "}
              must be validated before it becomes a BN254 scalar.
            </span>
          </li>
          <li className="flex gap-3">
            <span className="text-black dark:text-white font-bold shrink-0">2.</span>
            <span>
              A foreign extension-field value must be serialized into the byte
              order expected by the native pairing host functions.
            </span>
          </li>
        </ul>
      </section>

      <section className="mb-12">
        <h2 className="text-2xl font-bold text-black dark:text-white tracking-tight mb-4">
          Wrapper Shape
        </h2>
        <p className="text-neutral-600 dark:text-neutral-400 leading-relaxed mb-4">
          The exact limb radix is an implementation detail. A fixed-width
          wrapper keeps the raw value split into machine-size pieces until the
          boundary conversion is complete.
        </p>
        <CodeBlock
          code={`/// Conceptual shape for a foreign-field wrapper.
pub struct ForeignFieldElement<const LIMBS: usize> {
    pub limbs: [u64; LIMBS],
}`}
          language="rust"
          filename="foreign_field.rs"
        />
      </section>

      <section className="mb-12">
        <h2 className="text-2xl font-bold text-black dark:text-white tracking-tight mb-4">
          Boundary Conversion
        </h2>
        <p className="text-neutral-600 dark:text-neutral-400 leading-relaxed mb-4">
          The current public API exposes <code className="px-1.5 py-0.5 bg-neutral-100 dark:bg-neutral-800 rounded text-sm font-mono text-black dark:text-white">ZkEnv</code>{" "}
          and <code className="px-1.5 py-0.5 bg-neutral-100 dark:bg-neutral-800 rounded text-sm font-mono text-black dark:text-white">HostConvert</code>.
          Use them at the edge of the system, then move into canonical BN254
          types as early as possible.
        </p>
        <CodeBlock
          code={`use soroban_sdk::{Env, U256};
use zk_core::{Fr, ZkError};
use zk_soroban::{HostConvert, ZkEnv};

pub fn load_foreign_scalar(env: &Env, raw: U256) -> Result<Fr, ZkError> {
    if !env.is_bn254_scalar(raw) {
        return Err(ZkError::InvalidFieldElement);
    }

    env.fr_from_u256(raw)
}`}
          language="rust"
          filename="boundary.rs"
        />
      </section>

      <section className="mb-12">
        <h2 className="text-2xl font-bold text-black dark:text-white tracking-tight mb-4">
          Limb Arithmetic
        </h2>
        <p className="text-neutral-600 dark:text-neutral-400 leading-relaxed mb-4">
          If you need to operate on the wrapper directly, keep the arithmetic
          explicit: add limb-wise, propagate carries immediately, and reduce
          back to a unique canonical representative before the value leaves the
          wrapper.
        </p>
        <CodeBlock
          code={`const LIMB_BITS: u32 = 64;
const LIMB_MASK: u128 = u64::MAX as u128;

let mut carry: u128 = 0;
for i in 0..LIMBS {
    let sum = a.limbs[i] as u128 + b.limbs[i] as u128 + carry;
    out.limbs[i] = (sum & LIMB_MASK) as u64;
    carry = sum >> LIMB_BITS;
}

// Multiply with schoolbook accumulation, then reduce modulo the target prime.`}
          language="rust"
          filename="limbs.rs"
        />
      </section>

      <section className="mb-12">
        <h2 className="text-2xl font-bold text-black dark:text-white tracking-tight mb-4">
          Extension-Field Inputs
        </h2>
        <p className="text-neutral-600 dark:text-neutral-400 leading-relaxed mb-4">
          The same pattern shows up in G2 coordinates. `G2Affine` stores Fq2
          values as `(c0, c1)` pairs, and `pairing_check()` consumes the
          host-friendly serialization order for you.
        </p>
        <CodeBlock
          code={`pub struct G2Affine {
    pub x: (u256, u256),
    pub y: (u256, u256),
}

// x.1 before x.0, then y.1 before y.0
let bytes = g2.to_bytes();`}
          language="rust"
          filename="g2.rs"
        />
      </section>

      <section className="mb-12">
        <h2 className="text-2xl font-bold text-black dark:text-white tracking-tight mb-4">
          Integration Checklist
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {[
            "Keep one canonical limb order across the full code path.",
            "Validate every conversion against the target modulus.",
            "Return Err on out-of-range values; never panic.",
            "Convert into Fr or G2Affine before expensive arithmetic starts.",
          ].map((item) => (
            <div
              key={item}
              className="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900/50 text-sm text-neutral-600 dark:text-neutral-400"
            >
              {item}
            </div>
          ))}
        </div>
      </section>
    </DocsLayout>
  );
}
