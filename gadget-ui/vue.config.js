module.exports = {
    publicPath: '/_gadget/ui/',
    devServer: {
        port: 8082,
        proxy: {
            "^/_gadget/api": {
                target: "http://localhost:8080/"
            }
        },
    }
};