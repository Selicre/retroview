#version 140

in vec2 g_tex_coords;
flat in int g_pal;
out vec4 color;
uniform sampler2D tex;
uniform sampler2D palette;
uniform float water_level;

void main() {
	vec4 b_color = texture(tex, g_tex_coords);
	float px_id = floor(b_color.r*0x100);
	float pal = float(g_pal);
	if (gl_FragCoord.y < water_level) {
		pal += 4.0;
	}
	color = texture(palette, vec2(px_id/16.0 + 1.0/32.0, pal/16.0 + 1.0/32.0));
}
