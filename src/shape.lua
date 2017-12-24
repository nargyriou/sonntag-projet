local Object = require "classic"

local Shape = Object:extend()

function Shape:new()
	self.attached = {}
	return self
end

function Shape:attachShape(shape)
	table.insert(self.attached, shape)
end

function Shape:draw()
end

return Shape