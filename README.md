# mx002-driver-linux-settings

> You are in my README.md

It's strongly recomended to read the <a href = "./README - original.md"> Original Readme </a>.


## Introduction
Hi, I'm Ely Torres Neto, from Vertex Project.

Currently, I have a pen desk with mx002 chip, but unfortnally, we don't have that drivers for linux.
So, I looked up and found a project that solved my problem, but it's kinda hard code, good, but without a receptive interface for common users.

I decided to grab these driver and make my own version, forking from a project that drivers are already coded, adding a interface to change some configs, avoid direct changing things in code.

## Dependencies
- rustp
- cargo
- python3 and pip (check your linux distribution!)

## How to use

# Autors:
Ely Torres Neto - netoe1 (fork) - <a href = "github.com/netoe1">Github</a>

Marvin Belfort - marvinbelfort (original) <a href = "https://github.com/marvinbelfort">Github</a>

GMHadou (original) - <a href = "https://github.com/GMHadou">Github</a>

# Todo-List

## Main:
1) Change values from parameters (done)!
2) Add debug/status messages to terminal (done)!
3) Add support for JSON input (done, not testing)...
4) Add Interface in TK to communicate with JSON Input (todo)
5) Implement CLI for config (todo)

## Sub-main:
1) Caution with std::proccess:exit(-1), handle threads (todo)
2) Docs in code (doing)
3) Refactor old-way to get data, and make simple interface (todo)

