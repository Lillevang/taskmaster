# TaskMaster

TaskMaster is a terminal-based task management application built using Rust. It provides a clean and intuitive interface for managing tasks, complete with filtering, tagging, and detailed task views. TaskMaster aims to keep things simple while offering powerful features for effective task management.

## Features

- Two-Pane Layout: View your tasks on the left and detailed information on the right.
- Task Navigation: Quickly navigate through tasks using arrow keys.
- Task Details: View and edit task details, including descriptions, due dates, and tags.

## Installation

To build and run TaskMaster, you need to have Rust installed. You can install Rust using `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSF https://sh.rustup.rs | sh
```

Then, clone the repository and build the project:

```bash
git clone https://github.com/Lillevang/taskmaster.git
cd taskmaster
cargo build --release
```

## Usage

After building the project, you can run the application:

```bash
cargo run --release
```

### Key Bindings

- **Arrow Keys:**  Navigate through the task list.
- **Enter:** View or edit the selected task.
- **q:** Quit the program

## Contributing

Contributions are welcome! If you find a bug or have a feature request, please open an issue. Feel free to fork the repository and submit a pull request.

### Contributing Guidelines

- Please format your code using `rustfmt`.
- Ensure new code is covered by tests.
- Follow the existing coding style and patterns (unless addressing issues with these).

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Acknowledgments

- Built with Rust using the `ratatui`library for terminal UI.
