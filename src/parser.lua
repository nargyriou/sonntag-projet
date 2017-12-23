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
  uint32_t size;
  float (*ptr)[4];
} Rects;

Rects get_bounding_rects(const char *ptr);

void pretty_print(const char* input);
]]

local lib = ffi.load('target/debug/libelayr.' .. ext)
Parser.pretty_print = lib.pretty_print

function Parser:get_bounding_rects(input)
  local struct = lib.get_bounding_rects(input)

  local rects = {}

  for i=0,struct.size do
    local rect = struct.ptr[i - 1]
    table.insert(rects, Rectangle(
        rect[0] / 5 + 300, -- x
        rect[1] / 5 + 300, -- y
        rect[2] / 5, -- height
        rect[3] / 5 -- width
    ))
  end

  -- ffi.C.free(struct.ptr)

  return rects
end

return Parser
