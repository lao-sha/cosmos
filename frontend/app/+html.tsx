import { ScrollViewStyleReset } from 'expo-router/html';
import type { PropsWithChildren } from 'react';

export default function Root({ children }: PropsWithChildren) {
  return (
    <html lang="zh-CN" style={{ height: '100%' }}>
      <head>
        <meta charSet="utf-8" />
        <meta httpEquiv="X-UA-Compatible" content="IE=edge" />
        <meta
          name="viewport"
          content="width=device-width, initial-scale=1, shrink-to-fit=no, viewport-fit=cover"
        />
        <ScrollViewStyleReset />
        <style dangerouslySetInnerHTML={{ __html: `
          html, body {
            height: 100%;
            margin: 0;
            padding: 0;
            overflow: hidden;
          }
          #root {
            display: flex;
            flex-direction: column;
            height: 100%;
            min-height: 100vh;
          }
        `}} />
      </head>
      <body style={{ height: '100%', margin: 0, padding: 0, overflow: 'hidden' }}>
        {children}
      </body>
    </html>
  );
}
