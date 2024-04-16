# Contributing Guidelines

All contributions to the project are welcome! Before you start working with the project, we ask that you please read this document and follow the guidelines within.

## Table of contents

- [Setting up the development environment](#setting-up-the-development-environment)
- [Running tests](#running-the-tests)
- [Submitting a pull request](#submitting-a-pull-request)
  - [After submitting](#after-submitting)
- [Commit Message Guidelines](#commit-message-guidelines)

- Take a look at [existing issues](https://github.com/pltx/tui/issues).
  - If you need to create a new issue:
    - Make sure to use a clear and descriptive title.
    - Include as much information as possible: Steps to reproduce the issue, error message, version, operating system, etc.
    - Include the Operating System you are using.

## Setting up the development environment

### Prerequisites

- You have cloned the repository.
- [Rust](https://www.rust-lang.org/) is installed.

### Development

To run the project, run the following command:

```bash
cargo run
```

## Running the tests

To run the tests, run the following command:

```bash
cargo test
```

## Submitting a pull request

- Use a clear and descriptive title for the pull request.
- Describe the purpose of the pull request in the description.
- Reference any related issues and prs in the description.
- Include any necessary tests for the changes you have made.

### Commit Message Guidelines

We have very precise rules over how our git commit messages can be formatted. This leads to **more readable messages** that are easy to follow when looking through the **project history**.

#### Commit Message Format

Each commit message consists of a **header**, a **body** and a **footer**. The header has a special format that includes a **type**, a **scope** and a **subject**:

```
<type>[(scope)]: <subject>
<BLANK LINE>
[body]
<BLANK LINE>
[footer]
```

The **header** is mandatory and the **scope** of the header is optional.

Any line of the commit message cannot be longer than **100 characters**! This allows the message to be easier to read on GitHub as well as in various git tools.

#### Revert

If the commit reverts a previous commit, it should begin with `revert: `, followed by the header of the reverted commit. In the body it should say: `This reverts commit <hash>.`, where the hash is the SHA of the commit being reverted.

#### Type

Must be one of the following:

- **build**: Changes that affect the build system or external dependencies (example scopes: cargo, npm)
- **ci**: Changes to our CI configuration files and scripts (example scopes: Travis, Circle, BrowserStack, SauceLabs)
- **docs**: Documentation only changes
- **feat**: A new feature
- **fix**: A bug fix
- **perf**: A code change that improves performance
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- **test**: Adding missing tests or correcting existing tests

#### Scope

The scope could be anything specifying place of the commit change. For example `parser`, `lexer`, `compiler`, `runtime`, etc...

#### Subject

The subject contains succinct description of the change:

- use the imperative, present tense: "change" not "changed" nor "changes"
- don't capitalize first letter
- no dot (.) at the end

#### Body

Just as in the **subject**, use the imperative, present tense: "change" not "changed" nor "changes". The body should include the motivation for the change and contrast this with previous behavior.

#### Footer

The footer should contain any information about **Breaking Changes** and is also the place to reference GitHub issues that this commit **Closes**.

A detailed explanation can be found in this [document](https://www.conventionalcommits.org/en/v1.0.0/).
