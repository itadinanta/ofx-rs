print("Rendering test image...")
source = app1.createNode("net.sf.openfx.CheckerBoardPlugin")
mask = app1.createNode("net.sf.openfx.CheckerBoardPlugin")

under_test = app1.createNode("net.itadinanta.ofx-rs.simple_plugin_1")
under_test.connectInput(0, source)
under_test.connectInput(1, mask)

under_test.getParam("scale").setValue(0.5)

write = app1.createWriter("target/filtered_test_####.png")
write.connectInput(0, under_test)

app1.render(write, 1, 1)

quit()

