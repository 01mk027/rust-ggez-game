# Introduction
<p>Rust is my favourite programming language which is also most of software developers' favourite, enables developing incredible and memory-safe applications. One of the ways of learning Rust is developing projects with it, and as a eager learner, I am aware of this fact. So that, I've decide to simple project and publish it.</p>

<p>Aim of this project is to develop 2-D game which listens serial port to receive commands and performing some actions instead of using keyboard to perform some actions. In this game, user is responsible to destroy falling objects by hitting them with small bullets, format of this game is very known and common.</p>

<p>Arduino was used to guide the main character in this game. Aim of this approach is to learn embedded programming and performing information exchange between a microcontroller and a computer. This was very effective first step for this purpose.</p>

# Overview
<p>This game developed with  <a href="https://ggez.rs/">Ggez game engine</a>. Except bullets, all of objects are constructed as rectangle, because i am not so experienced on game development, this is my first project on game development.</p>

<p>Every falling objects from top of canvas has specific amount of hp in terms of numbers. If specific amount of bullets hit falling objects, object will disappeared with so so very basic animation. If falling object touches main character, game is terminated.</p>

<p>In this game, first time i have tried to use concurrent programming. Purpose is performing both listening serial port and increasing game speed. 
This game is programmed functionally, because i am beginner Rust programmer.</p>

# Used libraries
- chrono = "0.4.37"
- crossbeam-utils = "0.8.19"
- ggez = "0.9.3"
- lazy_static = "1.4.0"
- macroquad = "0.4.5"
- rand = "0.8.5"
- regex = "1.10.3"
- serde = { version = "1.0.197", features = ["derive"] }
- serde_json = "1.0"

# How i can install this game?
<b>This game is developed on Ubuntu 20.04.6 Focal Fossa.</b>
<p>First step, you need to complete installation steps on <a href="https://blog.logrocket.com/complete-guide-running-rust-arduino/">this site</a>. </p>

<p>After that, if installation steps are completed, you must run this command</p>

```
cargo build
```
<b>Don't forget to create folder named as resources  under target/debug destination and download "boom.ogg" and "pew.ogg" from <a href="https://github.com/ggez/ggez/tree/master/resources">here</a> and add into target/debug/resources destination.</b>
After that, you must clone <a href="#">this repository</a> and build circuit in same repository with respect to relevant repository and connect your Arduino to correct port.