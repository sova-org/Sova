# Multiplayer

Sova is built for collaborative live coding. Multiple players can connect to
the same server, see each other's code in real time, and perform together on a
shared scene.

## Starting a server

There are two ways to host a session:

- **Built-in server**: Launch the Sova desktop app, open the Server panel, and
  click Start. The app runs a server internally and connects to it. Other
  players can connect to your machine's IP address and port.
- **Standalone server**: Run `sova-server` from the command line with a port
  number. This is useful for dedicated hosting on a machine that doesn't need
  a GUI.

The server manages the scene, the clock, and all device connections. Clients
are lightweight — they send edits and receive updates.

## Connecting

To join a session:

1. Open the Server panel.
2. Enter the server's IP address and port.
3. Choose a username (must be unique in the session).
4. Click Connect.

Once connected, you receive the full scene, device configuration, and clock
state. You're immediately in sync with everyone else.

If the username is already taken or the connection is refused, you'll see an
error message. Pick a different name and try again.

## Peer editing

When multiple players are connected:

- You can see where each player's cursor is in the grid. Each player gets a
  distinct indicator on the cell they're viewing or editing.
- When someone starts editing a frame (opens the step editor), other players
  see that the frame is being edited. This helps avoid conflicting edits.
- All scene changes — adding lines, modifying frames, changing durations — are
  broadcast to every connected client in real time.

There is no locking: two players can edit different frames simultaneously
without conflict. If two players edit the same frame, the last evaluation wins.

## Chat

The Chat panel lets you send text messages to everyone in the session. Open it
from the panel menu or the context menu on the grid. Messages show the sender's
username.

## Scene synchronization

The server is the source of truth. When you evaluate code, add a frame, or
change a property, your edit is sent to the server, which applies it and
broadcasts the result to all clients. This means:

- Everyone always sees the same scene state.
- If you disconnect and reconnect, you get the current scene, not your last
  local state.
- The server's clock (via Ableton Link) keeps everyone time-aligned.

## Tips

- Agree on device slot assignments with your collaborators. If player A uses
  slot 1 for synth and player B reassigns slot 1 to drums, things will get
  confusing.
- Use different lines for different players to avoid stepping on each other's
  code.
- The chat is handy for coordinating transitions — "I'll drop the bass on the
  next quantum" — without breaking the flow of performance.
- Ableton Link synchronization works across the network, so even if players
  are on different machines, the beat stays locked.
