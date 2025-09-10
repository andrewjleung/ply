# ply

> 'Cause it's all getting flushed down the toilet anyways...

`ply` is my personalized job application tracking CLI, storing and managing job applications in plaintext with Markdown/TOML.

It exposes a simple CLI for:

1. Tracking new applications: `ply to <URL>`
2. Advancing existing applications: `ply yes <PATH> <NEXT_STAGE> [DEADLINE]`
3. Terminating existing applications: `ply no <PATH>`

## Tracking

All new applications are tracked in a configurable directory as Markdown files with TOML frontmatter capturing attributes like company, title, team, salary, stages, etc.

This allows you to track structured attributes for applications as well as whatever free-form notes you want to take throughout the application process.

Job listing data is automatically scraped from the given URL so long as it is HTTPS and for a supported job board (and provided that the parsing doesn't break due to changes in markup structure 😭).

The original job listing with its description is also converted into Markdown and saved within a separate directory, currently identifiable in its filename by a SHA256 hash of its URL.

## Motivation

Tracking applications with clicks and GUIs (Notion 😡) was pissing me off so I decided to just roll my own tracker.

`ply` aims to be a tool for me. It prioritizes:

1. **Instantaneousness**: track/update an application in as few steps as possible
2. **Programmability**: both in terms of implementing CLI features and interfacing it with other command-line utilities like `fzf`
3. **Simplicity**: applications are just simple text documents
4. **Hoarder-satisfaction**: keep track of every step of every application and the details of each job listing

## Job Board Support

To support the widest amount of job listings with the least amount of effort, `ply` primarily supports automated scraping from [`HiringCafe`](https://hiring.cafe). I'll be adding support to other boards as necessary, but for now `ply` also supports:

- Ashby

(yeah that's it 😬)

## Future Work

- Insights and application stats (number of apps, Sankey generator, bottlenecks, etc.)
- Grouping applications into custom cycles e.g. "post-grad", "senior software engineer 2025"
- Remote file store + API for tracking on-the-go
