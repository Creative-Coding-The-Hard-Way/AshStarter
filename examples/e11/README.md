# Example 10 - Compute Shader

This example is a bare-minimum demonstration of using a compute 
pipeline to write data from the GPU. 

To that end it is very *over synchronized* in that every compute 
submission and every draw is followed by a `device.wait_idle` call.
This destroys the framerate, but makes it trivial to ensure 
there are no data races.

## Commands

From the project root: `cargo run --example e10`

## Screenshot

![./Screenshot.jpg](./Screenshot.jpg)
