"use client";

import React, { useState } from "react";
import { Navbar } from "./Navbar";
import { Sidebar } from "./Sidebar";
import { PrevNextNav } from "./PrevNextNav";

interface DocsLayoutProps {
  children: React.ReactNode;
}

export function DocsLayout({ children }: DocsLayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="min-h-screen bg-white dark:bg-neutral-950 transition-colors duration-300">
      {/* Top Navbar */}
      <Navbar onToggleSidebar={() => setSidebarOpen(!sidebarOpen)} />

      {/* Content Area with Sidebar */}
      <div className="flex">
        {/* Sidebar Navigation */}
        <Sidebar open={sidebarOpen} onClose={() => setSidebarOpen(false)} />

        {/* Main Content */}
        <main className="flex-1 min-w-0">
          <div className="max-w-4xl mx-auto px-6 md:px-8 py-10 md:py-14">
            {children}
            <PrevNextNav />
          </div>

          {/* Footer */}
          <footer className="border-t border-neutral-200 dark:border-neutral-800 transition-colors duration-300">
            <div className="max-w-4xl mx-auto px-6 md:px-8 py-8 flex flex-col sm:flex-row justify-between items-center gap-4 text-sm text-neutral-400 dark:text-neutral-500">
              <span>© 2026 Soroban-ZK-Std</span>
              <a
                href="https://github.com/georgegoldman/Soroban-ZK-Std"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:text-black dark:hover:text-white transition-colors duration-200 tracking-wider uppercase text-xs font-semibold"
              >
                Edit this page on GitHub
              </a>
            </div>
          </footer>
        </main>
      </div>
    </div>
  );
}
