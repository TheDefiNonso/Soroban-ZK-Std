"use client";

import React, { useEffect, useState } from "react";

interface TocItem {
  id: string;
  text: string;
  level: number;
}

export function TocSidebar() {
  const [items, setItems] = useState<TocItem[]>([]);
  const [activeId, setActiveId] = useState<string>("");

  useEffect(() => {
    // Wait a brief moment to ensure MDX has rendered
    const timeoutId = setTimeout(() => {
      const elements = Array.from(document.querySelectorAll("main h2, main h3"));
      const headings = elements
        .map((elem) => ({
          id: elem.id,
          text: elem.textContent || "",
          level: Number(elem.tagName.charAt(1)),
        }))
        .filter((h) => h.id);

      setItems(headings);

      const observer = new IntersectionObserver(
        (entries) => {
          const visibleEntries = entries.filter((entry) => entry.isIntersecting);
          if (visibleEntries.length > 0) {
            // Find the first visible one
            setActiveId(visibleEntries[0].target.id);
          }
        },
        { rootMargin: "-100px 0px -66% 0px" }
      );

      elements.forEach((elem) => observer.observe(elem));

      return () => observer.disconnect();
    }, 100);

    return () => clearTimeout(timeoutId);
  }, []);

  if (items.length === 0) return null;

  return (
    <aside className="hidden xl:block w-64 shrink-0 pr-8 py-10">
      <div className="sticky top-24">
        <h4 className="text-sm font-semibold uppercase tracking-wider text-neutral-900 dark:text-neutral-100 mb-4">
          On this page
        </h4>
        <ul className="space-y-2.5 text-sm">
          {items.map((item) => (
            <li key={item.id} className={`${item.level === 3 ? "ml-4" : ""}`}>
              <a
                href={`#${item.id}`}
                className={`block truncate transition-colors duration-200 ${
                  activeId === item.id
                    ? "text-blue-600 dark:text-blue-400 font-medium"
                    : "text-neutral-500 dark:text-neutral-400 hover:text-black dark:hover:text-white"
                }`}
              >
                {item.text}
              </a>
            </li>
          ))}
        </ul>
      </div>
    </aside>
  );
}
