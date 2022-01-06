#version 300 es

precision mediump float;

in vec2 translation;
in float start_z;
in float speed;

uniform float PARTICLE_SPAWN_Z;
uniform float time_elapsed;
uniform float intensity_fraction;
uniform vec2 shake;

out float age;

void main() {
	float avg_speed = intensity_fraction * (speed + 2.) + (1. - intensity_fraction) * speed;
	float z = start_z - (time_elapsed * avg_speed);
	z = mod(z, PARTICLE_SPAWN_Z);
	age = 1. - z / PARTICLE_SPAWN_Z;

	gl_Position = vec4(translation, 0.0, z);
	gl_Position.xy += shake;
}
