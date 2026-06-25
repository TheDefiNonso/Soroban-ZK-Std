import type { Metadata } from "next";
import "./globals.css";
import { ThemeProvider } from "@/components/theme-provider";

const SITE_URL =
  process.env.NEXT_PUBLIC_SITE_URL?.replace(/\/$/, '') ??
  'https://soroban-zk-std.dev';

export const metadata: Metadata = {
  title: "Soroban-ZK-Std Docs",
  description: "The Zero-Knowledge Standard for Stellar",
  alternates: {
    types: {
      'application/rss+xml': [
        {
          url: `${SITE_URL}/rss.xml`,
          title: 'Soroban-ZK-Std Changelog',
        },
      ],
    },
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    // suppressHydrationWarning is required by next-themes on the html tag
    <html lang="en" suppressHydrationWarning>
      <body>
        <ThemeProvider
          attribute="class"
          defaultTheme="system"
          enableSystem
          disableTransitionOnChange
        >
          {children}
        </ThemeProvider>
      </body>
    </html>
  );
}
