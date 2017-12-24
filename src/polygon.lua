local Shape = require "shape"
local Point  = require "point"
local Line   = require "line"

local Polygon = Shape:extend()

function Polygon:new(...)
	local args = {...}
	local j = 1
	self.shape = "Polygon"
	self.x = args[1]
	self.y = args[2]
	self.vertices = {}
	for i=1, #args, 2 do
		self.vertices[j] = Point(args[i], args[i+1])
		j = j+1
	end

	j = 1
	self.edges = {}
	for i=1, #self.vertices-1 do
		self.edges[j] = Line(self.vertices[i], self.vertices[i+1])
		j = j+1
	end
	local i = #self.edges+1
	self.edges[i] = Line(self.vertices[i], self.vertices[1])

	return self
end

function Polygon:add(x, y)
	self:move(self.x + x, self.y + y)
end

function Polygon:move(x, y)
	local offx = x - self.x
	local offy = y - self.y
	for i,v in pairs(self.edges) do
		v:add(offx, offy)
	end
	self.x = x
	self.y = y
end

function Polygon:draw()
	for i,v in pairs(self.edges) do
		v:draw()
	end
end

return Polygon