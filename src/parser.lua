local ffi = require('ffi')
local Rectangle = require 'rectangle'

local Parser = {}

local ext

if ffi.os == 'Linux' then
    ext = 'so'
else
    ext = 'dylib'
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

local lib = ffi.load('target/debug/libelayr.' .. ext)
Parser.pretty_print = lib.pretty_print

function Parser:parse(input)
  local struct = lib.parse(input)

  print(struct.size)

  for i=0,struct.size-1 do
    local group = struct.groups[i]
    print("Group", group.x, group.y, group.h, group.w)

    for j=0,group.size-1 do
      local polygon = group.polygons[j]
      print("  Polygon", polygon.size)
      
      for k=0,polygon.size-1 do
        local vertice = polygon.vertices[k]
        print("    ", vertice.x, vertice.y)
      end
    end
  end

  return {}
end

return Parser
