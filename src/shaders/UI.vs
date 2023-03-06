#version 460 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec2 a_texcoord;

out vec2 tex_coord;

uniform mat4 projection;

void main()
{
    gl_Position = projection * vec4(a_pos, 1.0);
    tex_coord = a_texcoord;
}