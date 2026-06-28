# 📊 Vortex Frontend: Real-Time Transaction Dashboard

This is the frontend dashboard I built to visualize the **Vortex** transaction engine in real time. 

When I was building the backend for the Solana Smart Transaction Infrastructure Bounty, I realized that terminal logs weren't enough. I needed a way to visually prove that my dual-send strategy, Geyser streaming, and AI-driven tipping were actually working with zero latency. 

So, I built this React/Vite dashboard to stream transaction states (`submitted`, `processed`, `confirmed`, `finalized`) with millisecond-precision latency deltas directly to the browser.

## ✨ Features

- **Live State Streaming**: Watches the backend Geyser stream and updates the UI the exact millisecond a block is processed on the TPU.
- **Latency Deltas**: Displays the exact time (in milliseconds) between transaction submission, block processing, and cluster confirmation to prove network health.
- **AI Tip Visualization**: Shows the exact tip recommended by the AI Agent and the reasoning behind it before submission.
- **Interactive Sender**: Allows you to simulate or execute real SOL transfers to watch the backend engine autonomously manage the lifecycle.

## 🛠 Prerequisites

- Node.js (v18+)
- npm or pnpm
- The Vortex backend must be running in API Mode (see the [root README](../README.md) for backend setup).

## 🚀 Getting Started

1. Navigate to the frontend directory:
   ```bash
   cd frontend
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Start the Vite development server:
   ```bash
   npm run dev
   ```

4. Open your browser and navigate to `http://localhost:5173`. 

*Note: Make sure your `.env` variables are correctly set in the root directory so the backend can serve data to this dashboard!*
