#version 300 es
precision mediump float;

uniform vec2 RESOLUTION;

out vec2 uv;

void main() {
	// Render a full screen quad using a single triangle.
	// https://stackoverflow.com/a/59739538
	vec2 vertices[3] = vec2[3](vec2(-1,-1), vec2(3,-1), vec2(-1, 3));
	gl_Position = vec4(vertices[gl_VertexID], 0, 1);
	uv = vec2(gl_Position.x * 0.5 * RESOLUTION.x / RESOLUTION.y, gl_Position.y * 0.5);
}
