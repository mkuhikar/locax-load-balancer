# Locax Load Balancer: A Rust Round-Robin Load Balancer
A lightweight, multi-threaded load balancer written in Rust. It distributes incoming HTTP requests across a list of configured upstream servers using a Round-Robin strategy.

## Prerequisites
Before running the application, ensure you have the following installed:

Rust & Cargo: You can install them via rustup:

```Bash

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
## Configuration
The load balancer reads server configurations from a JSON file.

Create a file named upstream.json in the root directory.

Add your backend server details in the following format:

```JSON

[
  {
    "ip": "127.0.0.1",
    "port": 9001,
    "active": true
  },
  {
    "ip": "127.0.0.1",
    "port": 9002,
    "active": true
  },
  {
    "ip": "127.0.0.1",
    "port": 9003,
    "active": true
  }
]
```
## How to Run
1. Start the Backend Servers (for testing)
To test the load balancer, you need "upstream" servers to handle the traffic. You can simulate these easily using Python.

Open 3 separate terminal tabs and run:

Terminal 1 (Server A):

```Bash

python3 -m http.server 9001
```
Terminal 2 (Server B):

```Bash

python3 -m http.server 9002
```
Terminal 3 (Server C):

```Bash

python3 -m http.server 9003
```
2. Start the Load Balancer
Open a new terminal in the project root and run:

```Bash

cargo run
```
The load balancer will start on 127.0.0.1:8080 .

## Testing
To verify the load balancer is distributing traffic correctly, use curl.

### Single Request
```Bash

curl http://127.0.0.1:8080
```
### Stress Test / Loop Verification
Run the following loop to send repeated requests and watch the traffic cycle through your servers. If you are using the Python servers above, you should see "Directory listing..." responses from different ports in your server logs.

```Bash

while true; do curl -s http://127.0.0.1:8080 ; sleep 0.5; done
```
You will see it cycling: 9001, 9002, 9003...

### The Kill Test

Go to Terminal 2 (Port 9002) and press Ctrl+C to kill it.

Watch the Rust Load Balancer Terminal. Within 3 seconds, you should see: Server 127.0.0.1:9002 is DOWN

Look at your Curl loop in Terminal 5. It will no longer try to hit 9002. It will skip 9002 and only bounce between 9001 and 9003.

### The Resurrection Test

Start the Python server on 9002 again.

Watch the Rust LB. You should see: Server 127.0.0.1:9002 is back UP

Traffic will immediately start flowing to 9002 again.