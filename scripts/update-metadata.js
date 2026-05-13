const fs = require('fs');
const path = require('path');

const version = process.env.VERSION;
const signature = process.env.SIGNATURE;
const url = process.env.URL;

const metadata = {
  version: version,
  notes: `Release ${version}`,
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: signature,
      url: url
    }
  }
};

fs.writeFileSync('update.json', JSON.stringify(metadata, null, 2));
