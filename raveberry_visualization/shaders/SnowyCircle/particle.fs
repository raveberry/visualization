#version 300 es

precision mediump float;

in float age;
in float rot;

uniform vec2 RESOLUTION;

out vec4 fragColor;

#define TWO_PI 6.28318530718

void main() {
	vec2 translated_uv = gl_PointCoord - 0.5;
	vec2 polar = vec2(atan(translated_uv.x, translated_uv.y), length(translated_uv));
	polar.s /= TWO_PI;
	polar.s += 0.5;
	polar.s += rot;
	polar.s = mod(polar.s, 1./6.) - 1./12.;

	float x = polar.t * cos(polar.s) / RESOLUTION.y * 1080.;
	float y = polar.t * sin(polar.s) / RESOLUTION.y * 1080.;

	float brightness = 1. - length(gl_PointCoord - 0.5) * 2. * 1080./RESOLUTION.y;
	brightness = max(0., brightness);
	float ring = cos(polar.s * TWO_PI * 3.) * sin(x * TWO_PI * 4. + TWO_PI * 1.2);
	float bar = 1. - abs(y) * 100.;
	brightness *= max(bar, ring);
	brightness = max(0., brightness);

	float age_factor = max(0., age);
	fragColor = vec4(vec3(brightness * age_factor), 1);
}
