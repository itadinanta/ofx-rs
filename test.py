print("Test in progress...")
mask = app1.createNode("fr.inria.openfx.Checkerboard")
source = app1.createNode("fr.inria.openfx.Checkerboard")
under_test = app1.createNode("net.itadinanta.ofx-rs.simple_plugin_1")
under_test.connectInput(0, source)
under_test.connectInput(1, mask)

write = app.createWriter("target/filtered_test_####.png")
write.connectInput(0, under_test)

app1.render(write, 1, 1, 1)
quit()

