#version 140

in vec2 position;
in int tile_id;
in int flip;
in int pal;

out vec2 v_tex_coords;
out int v_flip;
out int v_pal;

uniform vec2 tex_size;

void main() {
	vec4 pos = vec4(position, 0.0, 1.0);
	gl_Position = pos;
	v_flip = flip;
	v_pal = pal;
	int xx = (tile_id*8)%int(tex_size.x);
	int yy = (tile_id*8)/int(tex_size.x)*8;
	v_tex_coords = vec2(xx,yy);
}
