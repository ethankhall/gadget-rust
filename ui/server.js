'use strict';

const express = require('express');
var exphbs  = require('express-handlebars');
const fs = require('fs');
const yaml = require('js-yaml');
var bodyParser = require('body-parser');

// App
const app = express();

app.use(bodyParser.urlencoded({ extended: true })); // support encoded bodies

// Constants
const PORT = 8080;
const HOST = '0.0.0.0';

app.engine('handlebars', exphbs());
app.set('view engine', 'handlebars');

function makeid(length) {
  var result           = '';
  var characters       = 'abcdefghijklmnopqrstuvwxyz0123456789';
  var charactersLength = characters.length;
  for ( var i = 0; i < length; i++ ) {
     result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
}

app.get('/', function (req, res) {
    var doc = yaml.safeLoad(fs.readFileSync('test1.yaml', 'utf8'));
    var redirects = [];
    for (var redirect of doc['redirects']) {
      var new_redirect = { 'id': redirect['id'], 'destination': redirect['destination']};
      let split = redirect['alias'].split(':');
      new_redirect['type'] = split[2];
      new_redirect['alias'] = split[3];

      redirects.push(new_redirect);
    }
    res.render('home', {'redirects': redirects});
});

app.get("/delete/:id", function(req, res){
  var doc = yaml.safeLoad(fs.readFileSync('test1.yaml', 'utf8'));

  var index = 0;
  for (var redirect of doc['redirects']) {
    if (redirect['id'] == req.params.id) {
      doc['redirects'].splice(index, 1)
      break;
    }
    index++;
  }

  fs.writeFileSync('test1.yaml', yaml.dump(doc));

  res.redirect("/");
});

app.get("/new", function (req, res) {
    res.render('new');
});

app.post("/new", function (req, res) {
  var doc = yaml.safeLoad(fs.readFileSync('test1.yaml', 'utf8'));

  doc['redirects'].push({
      'alias': 'url:gadget:' + req.body.redirectType + ':' + req.body.alias, 
      'destination': req.body.destination, 
      'id': makeid(10)
  });

  fs.writeFileSync('test1.yaml', yaml.dump(doc));
  res.redirect("/");
});

app.listen(PORT, HOST);
console.log(`Running on http://${HOST}:${PORT}`);