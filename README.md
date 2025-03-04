# Rust High-Frequency Trading Component

**Overview**  
This project is a high-frequency trading component developed in Rust. It leverages an event-driven engine, WebSocket, and ring buffer to acquire, write, and distribute market data in real time. The component is designed to interface with multiple exchanges, such as Binance, by providing encapsulated API interfaces and data parsing. Additionally, it features an order book maintenance module to manage and update order book data efficiently.

---

**Features**

- **Event-Driven Architecture:** Utilizes an event-driven engine for robust and scalable real-time data handling.
- **WebSocket Integration:** Implements WebSocket for real-time market data acquisition.
- **Ring Buffer:** Efficiently leverages a ring buffer that operates lock-free and asynchronously in a Single Producer Single Consumer (SPSC) manner for optimal data storage and processing.

- **Exchange Adaptation:** Adapts to multiple exchanges (e.g., Binance) with dedicated interfaces for API communication and data parsing.
- **Order Book Maintenance:** Provides a component to maintain and update order book information.

---



**Prerequisites**
- CentOS
- rustc 1.85.0

**Installation and Usage**
1. Clone the Repository

   ```bash
   git clone https://your-repository-url.git
   cd eborhft

2. Build the Project

   Make sure you have Rust installed. Then run:

   ```bash
   cargo build --release

3. Run the Component

   Execute the compiled binary:

   ```bash
   cargo run --release
---
**Contributing**

Contributions are welcome! Please fork the repository and create a pull request with your proposed changes. Ensure your code adheres to our coding standards and includes relevant tests.
