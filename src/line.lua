local Shape = require "shape"
local Point = require "point"
local Line = Shape:extend()

function Line:new(p1, p2)
	self.shape = "Line"
	self.x = p1.x
	self.y = p1.y
	self.offx = p2.x - p1.x
	self.offy = p2.y - p1.y
	self.p1 = p1
	self.p2 = Point(p1.x + self.offx, p1.y + self.offy)

	return self
end

function Line:draw()
	return love.graphics.line(self.p1.x, self.p1.y, self.p2.x, self.p2.y)
end

function Line:add(offx, offy)
	--print("Line ", offx, offy)
	self:move(self.x + offx, self.y + offy)
end

function Line:move(newx, newy)
	self.p1:move(newx, newy)
	self.p2:move(newx+self.offx, newy+self.offy)
end

return Line