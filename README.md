# Ping-Pong example

This example shows a *clients-server* connection.
The clients send a message every second and the server responds to it.

You can run both client and server by several transports: `tcp`, `udp`, `ws` (*WebSocket*).

## Notes

Always make sure the port that the server will bind to is open to the network,
modern linux distributions and those who aren't minimal usually do this on their own temporarily
and then close it after the program is done, but linux distros like 4MLinux or TheSSS don't.
This caused me immense pain and cost me 5 hours to find out, fuck TheSSS, why can't you open ports
automatically
