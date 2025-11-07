# 🚀 grow.rs

A modern, fast markdown viewer server built with Rust and Axum. Transform your directory into a beautiful web-based documentation browser with live-reload capabilities.

## ✨ Features

- **📁 Directory Browsing**: Navigate through directories with a stunning modern UI
- **📝 Markdown Rendering**: Convert `.md` and `.mkd` files to beautiful HTML on-the-fly
- **🔄 Live Reload**: Automatic browser refresh when files change
- **🎨 Modern Design**: Sleek gradient backgrounds with glassmorphism effects
- **📱 Responsive**: Looks great on desktop and mobile devices
- **⚡ Fast**: Built with Rust for maximum performance
- **🔒 Secure**: Path traversal protection prevents unauthorized access
- **🌐 Smart Port Selection**: Automatically finds available ports

## 🛠️ Installation

### Prerequisites

You need to install [unidoc](https://github.com/cympfh/unidoc) for markdown processing:

```bash
# Install unidoc (markdown to HTML converter)
cargo install unidoc
```

### Install grow.rs

```bash
# Clone and build
git clone https://github.com/cympfh/grow.rs
cd grow.rs
cargo build --release

# Or install directly from source
cargo install --path .
```

## 🚀 Usage

### Basic Usage

```bash
# Serve current directory on default port (8080)
grow .

# Serve specific directory
grow /path/to/your/docs

# Specify port and host
grow --port 3000 --host 127.0.0.1 ./my-docs
```

### Command Line Options

- `--port <PORT>`: Set port number (default: 8080, auto-finds available port)
- `--host <HOST>`: Set host address (default: 0.0.0.0)
- `<DIRECTORY>`: Directory to serve (default: current directory)

### Example

```bash
$ grow --port 8080 ./documentation
Starting server on http://0.0.0.0:8080
```

Then open your browser and navigate to `http://localhost:8080`

## 🏗️ Architecture

grow.rs is built with a clean, modular architecture:

- **Axum Web Framework**: High-performance async web server
- **File Watching**: Real-time monitoring using the `notify` crate
- **Server-Sent Events**: Live reload functionality
- **External Processing**: Uses `unidoc` for markdown conversion
- **Concurrent Design**: File watching runs in separate threads
