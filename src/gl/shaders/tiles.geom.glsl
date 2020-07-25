#version 150 core
layout(points) in;
in vec2 v_tex_coords[1];
//in ivec3 v_flags[1];
//in int v_pal[1];

in int v_flip[1];
in int v_pal[1];
layout(triangle_strip, max_vertices=4) out;
out vec2 g_tex_coords;
flat out int g_pal;
uniform mat4 matrix;
uniform vec2 tex_size;

uniform float loop_height;
uniform float loop_width;

void main() {
	vec4 pos = gl_in[0].gl_Position;
	vec4 t = matrix * pos;
	vec4 t2 = matrix * vec4(pos.x+8.0, pos.y+8.0, pos.z, pos.w);
	if (t2.x < -1.0) {
		// simulate loopback
		pos.x += loop_width;
		pos.y -= 128;
	} else {
		if (t.y < -1.0) {
			// show on the bottom
			pos.y -= loop_height;
		}
		if (t2.y > 1.0) {
			// show on the top
			pos.y += loop_height;
		}
	}

	float left = 0.0;
	float right = 8.0;
	float top = 0.0;
	float bottom = 8.0;
	if ((v_flip[0] & 0x1) != 0) {
		left = 8.0;
		right = 0.0;
	}
	if ((v_flip[0] & 0x2) != 0) {
		top = 8.0;
		bottom = 0.0;
	}
	g_pal = v_pal[0];

	g_tex_coords = (v_tex_coords[0] + vec2(left, top)) / tex_size;
	gl_Position = matrix * (pos + vec4(0.0, 0.0, 0.0, 0.0));
	EmitVertex();

	g_tex_coords = (v_tex_coords[0] + vec2(right, top)) / tex_size;
	gl_Position = matrix * (pos + vec4(8.0, 0.0, 0.0, 0.0));
	EmitVertex();

	g_tex_coords = (v_tex_coords[0] + vec2(left, bottom)) / tex_size;
	gl_Position = matrix * (pos + vec4(0.0, 8.0, 0.0, 0.0));
	EmitVertex();

	g_tex_coords = (v_tex_coords[0] + vec2(right, bottom)) / tex_size;
	gl_Position = matrix * (pos + vec4(8.0, 8.0, 0.0, 0.0));
	EmitVertex();

	EndPrimitive();
}
