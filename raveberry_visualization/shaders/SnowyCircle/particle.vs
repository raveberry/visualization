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
out float rot;

float rand(float x) {
    return fract(sin(x * 91.2228) * 43758.5453);
}

void main() {
	float avg_speed = intensity_fraction * (speed + 1.5) + (1. - intensity_fraction) * speed;
	float y = translation.y - (time_elapsed * avg_speed);
	float z = start_z;
	float x = translation.x + 0.05 * sin(10. * y * speed) * (rand(z) - 0.5);
	x *= z;
	float MOD = 4.;
	y = mod(y + MOD/2., MOD) - MOD/2.;
	z = mod(z, PARTICLE_SPAWN_Z);
	age = 1. - z / PARTICLE_SPAWN_Z;
	rot = (rand(z) - 0.5) * y;

	gl_Position = vec4(x, y, 0.0, z);
	gl_Position.xy += shake;
}
