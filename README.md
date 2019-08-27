# Ray Tracing The Next Week

<img align="center" src="https://github.com/VoidPtr74/RayTracingTheNextWeek/blob/master/title_image.jpg">

## Overview

Yet another implementation of a ray-tracer based on Peter Shirley's "[Ray Tracing The Next Week](https://github.com/RayTracing/TheNextWeek)" written in Rust.

## Building

This implementation uses intrinsics for SSE instructions. To compile, run the following: 

`cargo rustc --release -- --C target-cpu=native`

## Running

For best performance, I recommend building for and running on a cpu that supports FMA AVX instructions. The picture at the top was rendered in 39.97 hours on an Intel i7-4790k CPU. The image was rendered at 3840x2160 with 65536 samples per pixel, running 24 worker threads with a maximum of 20 bounces per ray.

## Notes

The implementation for vec3 and enhancements to AABB were translated from [GPSnoopy](https://github.com/GPSnoopy)'s C++ implementation of [the next week](https://github.com/GPSnoopy/RayTracingTheNextWeek) 
