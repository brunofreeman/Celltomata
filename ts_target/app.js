"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const express = require("express");
const bodyParser = require("body-parser");
const cookieParser = require("cookie-parser");
exports.app = express();
exports.app.use(bodyParser.json());
exports.app.use(bodyParser.urlencoded({ extended: false }));
exports.app.use(cookieParser());
exports.app.use(express.static('website'));
exports.app.use('/', async (req, res) => {
    res.sendFile(__dirname + '/website/index.html');
});
// catch 404 and forward to error handler
exports.app.use(async (req, res, next) => {
    // const err = new Error('Not Found');
    res.end("Not found!");
});
//# sourceMappingURL=app.js.map