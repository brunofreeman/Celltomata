import * as express from 'express';
import * as bodyParser from 'body-parser';
import * as cookieParser from 'cookie-parser';
import * as path from 'path';

export let app = express();

app.use(bodyParser.json());
app.use(bodyParser.urlencoded({ extended: false }));
app.use(cookieParser());

app.use(express.static('website'));

app.use('/', async (req, res) => {
    res.sendFile(__dirname + '/website/index.html');
});

// catch 404 and forward to error handler
app.use(async (req, res, next) => {
    // const err = new Error('Not Found');
    res.end("Not found!")
});