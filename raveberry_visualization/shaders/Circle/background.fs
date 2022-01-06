#version 300 es

precision mediump float;

in vec2 uv;

uniform vec3 top_color;
uniform vec3 bot_color;

out vec4 fragColor;

void main() {
	vec3 color = mix(bot_color, top_color, -uv.y + 0.5);
	fragColor = vec4(color, 1);
}
