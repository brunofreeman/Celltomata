var canvas = document.getElementById("canvas");
var context = canvas.getContext("2d");

canvas.width  = window.innerWidth;
canvas.height = window.innerHeight;

var scale = 2;
var C = 16 * scale;
var R = 9 * scale;

context.lineWidth = 5;
context.strokeStyle = "black";

for (var c = 0; c < C; c++) {
    for (var r = 0; r < R; r++) {
        context.beginPath();
        context.rect(c * canvas.width / C, r * canvas.height / R, canvas.width / C, canvas.height / R);
        context.stroke();
    }
}

context.fillStyle = "green";

function fillSquare(event) {
    var x = event.clientX;
    var y = event.clientY;
    c = x - (x % (canvas.width / C));
    r = y - (y % (canvas.height / R));
    //console.log("(%d, %d) --> (%d, %d)", x, y, c, r);
    context.beginPath();
    context.rect(c, r, canvas.width / C, canvas.height / R);
    context.fill();
    context.stroke();
    
}