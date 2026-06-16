# Architecture Diagram

```mermaid
graph TD
    %% Core Services
    Core[Core Coordinator]
    Agent[AI Agent (Claude)]
    
    %% External Services
    Geyser[Yellowstone Geyser Stream]
    RPC[Solana RPC]
    Jito[Jito Block Engine]
    
    %% Internal Modules
    Leader[Leader Tracker]
    Logger[Lifecycle Logger]
    Failure[Failure Classifier & Retry]
    
    %% Data Flow
    Geyser -- Pushes live Slots & Tx Confirmations --> Core
    RPC -- Polling Fallback & Tip Stats --> Core
    
    Core -- 1. Predict Jito Window --> Leader
    Core -- 2. Request Tip Decision --> Agent
    Agent -- 3. AI Calculates Tip & Confidence --> Core
    
    Core -- 4. Construct & Sign Bundle --> Jito
    Jito -- 5. Broadcast to Validators --> RPC
    
    Geyser -- 6. Landing Detected (Zero Latency) --> Core
    Core -- 7. Record Timestamps & Latency --> Logger
    
    Core -- 8. On Error (e.g. Blockhash Expired) --> Failure
    Failure -- 9. Classify & Trigger AI Retry --> Core
    
    classDef external fill:#f9f,stroke:#333,stroke-width:2px;
    class Geyser,RPC,Jito,Agent external;
```
