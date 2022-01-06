#version 300 es
precision mediump float;

in vec2 uv;

uniform vec2 RESOLUTION;
uniform float BARS;
uniform float time_elapsed;
uniform float current_intensity;
uniform vec2 shake;
uniform vec3 recent_color;
uniform vec3 past_color;
uniform sampler2D spectrum;
uniform sampler2D logo;

out vec4 fragColor;

#define TWO_PI 6.28318530718
#define CIRCLE_RADIUS 0.24
#define FADE_DIST 0.005
#define SPECTRUM_PEAK 0.14
#define INTENSITY_SCALE 0.1

void main() {
	// rotating the coordinates would be easy, but the texture access would be costly
	// thus, we do not rotate.

	// shaking the cartesian uv is quite expensive, simplify by shaking the polar origin
	vec2 translated_uv = uv + shake;

	// Get polar coordinates
	vec2 polar = vec2(atan(translated_uv.x, translated_uv.y), length(translated_uv));

	// Scale to a range of 0 to 1
	polar.s /= TWO_PI;
	polar.s += 0.5;

	//
	// Spectrum
	//
	float fft_x = polar.s;
	// Mirror at 0.5
	// ....x    ..x..
	// ...x.    .....
	// ..x.. => .x.x.
	// .x...    .....
	// x....    x...x
	fft_x = -abs(2. * fft_x - 1.) + 1.;
	// Invert (low frequencies on top)
	fft_x = 1.0 - fft_x;

	float fft = texture(spectrum, vec2(fft_x, 0.)).r;
	float circle_base_radius = CIRCLE_RADIUS + current_intensity * INTENSITY_SCALE;
	float scaled_peak = (1. + INTENSITY_SCALE * 2. * current_intensity) * SPECTRUM_PEAK;
	float spectrum_position = (polar.t - circle_base_radius) / scaled_peak;
	float active_spectrum_fraction = spectrum_position / fft;

	const float white_border = 0.5;
	vec3 white = vec3(1);
	vec3 black = vec3(0);
	// blend the border of the spectrum with the background
	float alpha = 1. - smoothstep(fft - (1. + 2. * fft) * FADE_DIST * 10., fft, spectrum_position);
	float color_spectrum_fraction = (active_spectrum_fraction - white_border) * 1. / (1. - white_border);
	color_spectrum_fraction = clamp(color_spectrum_fraction, 0., 1.);
	vec3 rgb = mix(past_color, recent_color, color_spectrum_fraction);
	// blend the white portion with the colorful one
	rgb = mix(white, rgb, smoothstep(white_border - FADE_DIST * 50., white_border, active_spectrum_fraction));

	//
	// Logo
	//
	float logo_radius = circle_base_radius - FADE_DIST;

	// scale the logo and translate it into the middle of the screen
	vec2 logo_uv = uv + 0.5 + shake;
	float scale = 0.7 / logo_radius;
	logo_uv *= scale;
	logo_uv = logo_uv - 0.5 * (scale - 1.);

	// use the cartesian coordinates as fake normals for a spherical look
	vec3 normal = vec3(logo_uv.x - .5, -logo_uv.y + 0.5, 1);
	// increase curvature of the faked sphere
	normal.xy *= 0.5;
	normal = normalize(normal);
	vec3 light = normalize(vec3(1, -1, 1));
	vec3 reflected = normalize(2.0 * dot(normal, light) * normal - light);
	// center the logo_uv to get faked button like normals
	const float intensity = 0.75;
	const float shinyness = 8.;
	vec3 specular = vec3(1) * intensity * pow(max(0.0, dot(vec3(0, 0, 1), reflected)), shinyness);

	vec3 tex_color = texture(logo, logo_uv).rgb;
	rgb = mix(rgb, specular, 1. - smoothstep(logo_radius - FADE_DIST, logo_radius, polar.t));
	rgb += tex_color;

	//
	// Postprocessing
	//
	// Lighten the screen when there is high intensity
	float bright_alpha = current_intensity * INTENSITY_SCALE;
	// Vignette
	float circle_max_radius = CIRCLE_RADIUS + INTENSITY_SCALE + scaled_peak;
	float dark_alpha = 1. - smoothstep(0.0, 1.0, 1.5 - length(uv));
	rgb = mix(rgb, black, step(circle_max_radius, polar.t));
	alpha = mix(alpha, dark_alpha, step(circle_max_radius, polar.t));
	rgb += vec3(1) * bright_alpha;

	fragColor = vec4(rgb, alpha);
}

