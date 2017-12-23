local ffi = require('ffi')
local Rectangle = require 'rectangle'

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

local lib = ffi.load('../target/debug/libelayr.' .. ext)
local pretty_print = lib.pretty_print

function get_bounding_rects(input)
  local struct = lib.get_bounding_rects(input)

  local rects = {}

  for i=0,struct.size do
    local rect = struct.ptr[i - 1]
    table.insert(rects, Rectangle:new(
        rect[0], -- x
        rect[1], -- y
        rect[2], -- height
        rect[3] -- width
    ))
  end

  -- ffi.C.free(struct.ptr)

  return rects
end

return {
  get_bounding_rects,
  pretty_print
}
