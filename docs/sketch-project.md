# Sketch a Project - 101

## Idea
---
Create an CLI applicaiton to manage the claude resources, such as skills, agents, commands, hooks, and rules, making it easier for developer to setup the claude resources for their project. The claude resources can be coming from [external libraries](../extern/), [claude local user folder](/C/Users/username/.claude) or [claude project folder](/path/to/claude/project/folder).
What's more, the CLI application can also used to manage the claude resources used in a session. For example, what skills, hooks and commands (with permission) are used in a session, and what agents are used in a session. The session can be running in different patterns, such as running in a simple conversation, running in a agent loop, or running in interactive mode with human in the loop.

## Architecture
---
The architecture of the CLI application can be divided into three layers: the [CLI layer](../crates/cc-cli/), the [service layer](../crates/cc-core/), and the [data layer](../crates/cc-schema/). The data layer will store specific configurations for each local project, external libraries, and local user folder. The schema will be defined in the format of json or rust easy access struct.

### Architecture docs
---
The architecture document require the following sections:
1. **Overview**: A high-level description of the architecture, including the main components and their interactions.
2. **Components**: A detailed description of each component, including its responsibilities, interfaces, and interactions with other components. Compare different solutions for each component side by side in a table format, and explain the pros and cons of each solution.
3. **Data Flow**: A description of how data flows through the system, including any data transformations that occur.
4. **Error Handling**: A description of how errors are handled in the system, including any retry mechanisms or fallback strategies.
5. **Testing**: A description of the testing strategy for the system, including any unit tests, integration tests, or end-to-end tests that will be implemented.
6. **Deployment**: A description of how the system will be deployed, including any infrastructure requirements and deployment strategies.
7. **use cases**: A description of the main use cases for the system, including any user stories or scenarios that will be supported.
