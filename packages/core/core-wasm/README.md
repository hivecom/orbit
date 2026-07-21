# core-wasm

This library builds upon core-shared to provide an easy-to-use IRC interface. To
simplify server communication, the connection is abstracted into a stateful
actor. When the actor receives a matching reply from the IRC server, it routes
it back to the sender via oneshot channels. Additionally, it allows you to
configure callbacks for events, errors, and disconnections.

```mermaid
sequenceDiagram
participant wasm as WASM Interface
box Actor
participant M as Actor (state)
participant C as IrcConnection
end

rect rgb(191, 223, 255)
note right of wasm: await server.join("#35;channel")
wasm->>M: Send message: Join("#35;channel")

M->>C: JOIN #35;channel
C-->>M: JOIN #35;channel

M->>M: update state
M-->>wasm: Send response via Oneshot
end

wasm-->>M: Send message: Register event callback
C->>M: PRIVMSG Alice #35;channel :hello!
M->>M: update state
M-->>wasm: Send to registered event callbacks
```
