# Testing Strategy

## Model Context Protocol Server Mocking
- Roadmap Server: This server responsibility for all tool relate with roadmap.
- Lesson Server: This server responsibility for all tool relate with lesson.
- Quiz Server: This server responsibility for all tool need to generate quiz for lesson.
- Practice Server: This server responsibility for all tool need to generate practice for lesson.
- Database Server: This server responsibility for all tool to store anything to Database.
- External Server: Other server don't relate with our ecosystem, those server is helping user can visuallize lesson(Figma MCP), find many practice relate with skill(Github MCP).

## Tools Mocking
#### Roadmap Server
- extract_topic: Extract that topic receive -> return list key concept.
- generate_individual_roadmap: Generate one roadmap for every topic -> return roadmap schema.
    * Example: 
        + Topic: Rust Basic -> roadmap of Rust Fundamental.
        + Topic: Rust Backend -> Rust Fundamental - Backend Fundamental - Rust Backend.
- generate_dual_roadmap: Generate many roadmap for many topic.
    * Example:
        + Topic: Rust Basic -> roadmap of Rust Fundamental.
        + Topic: Rust Backend -> ["Rust Fundamental", "Backend Fundamental"].

#### Lesson Server
- core_lesson_generate: Knowledge need for Chapter.
    * Example: Rust Fundamental -> ["Chapter 1", "Chapter 2"] (roadmap)
            core_lesson_tool:
                - Chapter 1: Knowledge.
                - Chapter 2: Knowledge.
            -> Vec<HashMap<String, String>>
- module_lesson_generate: Generate details knowledge of parent knowledge.
    * Example:
        + Chapter 1: Knowledge -> Module 1.1, Module 1.2, Module 1.3, ...
        + Module 3.1: Knowledge of Module 3.1 -> Module 3.1.1, Module 3.1.2, Module 3.1.3, ...

#### Quiz Server
- chapter_quiz_tool: Quiz to test knowledge of user.

#### Practice Server
- practice_tool: Practice to test skill of user.

#### Database Server
- infomation_tool: CRUD infomation of user.
- lesson_db_tool: CRUD lesson.
- roadmap_db_tool: CRUD roadmap.
- practice_db_tool: CRUD practice.

#### External Server
Many External server: Github MCP, Figma MCP, Notion MCP.

## Strategy
when prompt have span ```Testing: user prompt``` that is enable for testing, using all tool mocking that describe to plan and execute.