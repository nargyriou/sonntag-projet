local ffi = require('ffi')
local Rectangle = require 'rectangle'
local Polygon   = require 'polygon'

local Parser = {}

local file
if ffi.os == 'Windows' then
	file = 'elayr.dll'
elseif ffi.os == 'Linux' then
	file = 'libelayr.so'
else
	file = 'libelayr.dylib'
end

ffi.cdef[[
typedef struct {
	float x;
	float y;
} Point;

typedef struct {
	uint32_t size;
	const Point *vertices;
} Polygon;

typedef struct {
	float x;
	float y;
	float h;
	float w;
	uint32_t size;
	const Polygon *polygons;
} Group;

typedef struct {
	uint32_t size;
	const Group *groups;
} Drawing;

Drawing parse(const char* input);

void pretty_print(const char* input);
]]

local lib = ffi.load('target/release/' .. file)
Parser.pretty_print = lib.pretty_print

function Parser:parse(input)
	local struct = lib.parse(input)
	local rects = {}

	-- print(struct.size)

	for i=0,struct.size-1 do
		local group = struct.groups[i]
		-- print("Group", group.x, group.y, group.h, group.w)
		-- FIXME
		local rect = Rectangle(group.x / 5, group.y / 5, group.h / 5, group.w / 5)
		table.insert(rects, rect)

		for j=0,group.size-1 do
			local polygon = group.polygons[j]
			-- print("  Polygon", polygon.size)
			
			local vertices = {}
			for k=0,polygon.size-1 do
				table.insert(vertices, polygon.vertices[k].x / 5)
				table.insert(vertices, polygon.vertices[k].y / 5)
			end
			rect:attachShape(Polygon(unpack(vertices)))
		end
	end

	return rects
end

return Parser
