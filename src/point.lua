local Shape = require "shape"

local Point = Shape:extend()

function Point:new(x, y)
	self.shape = "Point"
	if x and y then
		self.x = x
		self.y = y
		return self
	else
		return nil
	end
end

function Point:add(x, y)
	self:move(self.x + x, self.y + y)
end

function Point:move(x, y)
	self.x = x
	self.y = y
end

function Point:explode()
	return x, y
end

function Point:draw()
	return love.graphics.circle("fill", self.x, self.y, 3, 4)
end

return Point