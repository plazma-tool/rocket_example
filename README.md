# GNU Rocket example project

A [GNU Rocket Editor][rocket] example project in Rust.

See the article, [Let's try 0.5 to 1.5][article] for details, with an oveview of
the client- and sync libs, and this example project.

To get this going on your machine, clone the example repo and compile. Enable
the `env_logger` messages, it can be useful.

~~~ bash
git clone https://github.com/make-a-demo-tool-in-rust/rocket_example
cd rocket_example
RUST_LOG=rocket_example,rocket_sync cargo run
~~~

To see the animation, you will also need to install Rocket and open
`data/tracks.rocket` with it.

![rocket example](images/rocket-example-demo.gif)

[article]: https://make-a-demo-tool-in-rust.github.io/2-0-lets-try-0-5-to-1-5.html
[rocket]: https://github.com/emoon/rocket
