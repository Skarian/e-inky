import Head from 'next/head';

export default function Home() {
  return (
    <>
      <Head>
        <title>e-inky Library Manager</title>
        <meta
          name="description"
          content="Desktop companion for preparing and syncing content to the Xteink X4."
        />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
      </Head>
      <main className="container">
        <section className="card">
          <h1>e-inky</h1>
          <p>
            This Tauri + Next.js app will convert EPUBs into the X4&apos;s native XTC format and
            synchronize your library to the device&apos;s microSD card.
          </p>
          <p>
            The project is set up with TypeScript, ESLint, and Tauri commands so you can focus on
            implementing the conversion and sync pipeline.
          </p>
          <ul>
            <li>Run <code>pnpm dev</code> to work on the Next.js UI.</li>
            <li>Run <code>pnpm tauri:dev</code> to start the desktop shell with hot reload.</li>
            <li>Run <code>pnpm tauri:build</code> to package the application.</li>
          </ul>
        </section>
      </main>
    </>
  );
}
