#version 300 es

precision mediump float;

in float age;

uniform vec2 RESOLUTION;

out vec4 fragColor;

#define TWO_PI 6.28318530718

void main() {
	// gl_PointSize is not supported on the Pi, so we fake farther particles being smaller here
	// we also adjust for smaller resolutions
	float age_size = 2. - age;
	float resolution_correction = 1080./RESOLUTION.y ;
	float sharpness = 2.;
	float brightness = sharpness * (1. - length(gl_PointCoord - 0.5) * 2. * resolution_correction * age_size);
	brightness = max(0., brightness);

	// decrease the brightness due to additive blending
	brightness *= 0.75;

	// age the particles as fast as possible so that they spawn behind the border of the ring
	const float aging_speed = 1.2;
	float age_factor = age * aging_speed - (aging_speed - 1.);
	age_factor = max(0., age_factor);
	fragColor = vec4(vec3(brightness * age_factor), 1);
}
