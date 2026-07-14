import argparse
import hashlib
import json
import socket
from pathlib import Path


SENSITIVE_HEADERS = {"authorization", "x-api-key"}


def receive_request(port: int, output: Path, expected_sha256: str) -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        listener.bind(("127.0.0.1", port))
        listener.listen(1)
        listener.settimeout(30)
        connection, peer = listener.accept()

        with connection:
            connection.settimeout(30)
            received = bytearray()
            receive_sizes = []
            while b"\r\n\r\n" not in received:
                chunk = connection.recv(65536)
                if not chunk:
                    raise RuntimeError("connection closed before request headers completed")
                received.extend(chunk)
                receive_sizes.append(len(chunk))

            raw_head, body = bytes(received).split(b"\r\n\r\n", 1)
            lines = raw_head.decode("iso-8859-1").split("\r\n")
            request_line = lines[0]
            headers = []
            content_length = None
            transfer_encoding = None
            for line in lines[1:]:
                name, value = line.split(":", 1)
                value = value.lstrip()
                lower_name = name.lower()
                if lower_name == "content-length":
                    content_length = int(value)
                elif lower_name == "transfer-encoding":
                    transfer_encoding = value
                headers.append(
                    [name, "<redacted>" if lower_name in SENSITIVE_HEADERS else value]
                )

            if content_length is None:
                raise RuntimeError(
                    f"probe currently requires Content-Length, got Transfer-Encoding={transfer_encoding!r}"
                )
            while len(body) < content_length:
                chunk = connection.recv(min(65536, content_length - len(body)))
                if not chunk:
                    raise RuntimeError("connection closed before request body completed")
                body += chunk
                receive_sizes.append(len(chunk))
            body = body[:content_length]
            body_sha256 = hashlib.sha256(body).hexdigest().upper()

            summary = {
                "peer": list(peer),
                "request_line": request_line,
                "headers": headers,
                "content_length": content_length,
                "transfer_encoding": transfer_encoding,
                "body_bytes": len(body),
                "body_sha256": body_sha256,
                "body_matches_expected": body_sha256 == expected_sha256.upper(),
                "socket_receive_sizes": receive_sizes,
            }
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text(json.dumps(summary, indent=2), encoding="utf-8")
            connection.sendall(
                b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nOK"
            )

    print(json.dumps(summary, separators=(",", ":")))


def send_httpx(url: str, body_path: Path, request_id: str) -> None:
    import httpx

    body = body_path.read_bytes()
    placeholder = "local-capture-placeholder"
    headers = {
        "Accept": "application/json",
        "Authorization": f"Bearer {placeholder}",
        "Content-Type": "application/json",
        "anthropic-dangerous-direct-browser-access": "true",
        "anthropic-version": "2023-06-01",
        "User-Agent": "claude-cli/2.1.92 (external, sdk-cli)",
        "x-app": "cli",
        "x-claude-code-session-id": "576a7d1a-0573-45fb-8359-7d360ae1b40a",
        "x-client-request-id": request_id,
        "x-stainless-arch": "x64",
        "x-stainless-lang": "js",
        "x-stainless-os": "Windows",
        "x-stainless-package-version": "0.80.0",
        "x-stainless-retry-count": "0",
        "x-stainless-runtime": "node",
        "x-stainless-runtime-version": "v24.3.0",
        "x-stainless-timeout": "600",
        "x-api-key": placeholder,
    }
    with httpx.Client(http1=True, http2=False, trust_env=False, timeout=30) as client:
        response = client.post(url, headers=headers, content=body)
        print(
            json.dumps(
                {
                    "status": response.status_code,
                    "http_version": response.http_version,
                    "body_bytes": len(body),
                    "body_sha256": hashlib.sha256(body).hexdigest().upper(),
                },
                separators=(",", ":"),
            )
        )


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)

    server = subparsers.add_parser("server")
    server.add_argument("--port", type=int, required=True)
    server.add_argument("--output", type=Path, required=True)
    server.add_argument("--expected-sha256", required=True)

    sender = subparsers.add_parser("send-httpx")
    sender.add_argument("--url", required=True)
    sender.add_argument("--body", type=Path, required=True)
    sender.add_argument("--request-id", required=True)

    args = parser.parse_args()
    if args.command == "server":
        receive_request(args.port, args.output, args.expected_sha256)
    else:
        send_httpx(args.url, args.body, args.request_id)


if __name__ == "__main__":
    main()
