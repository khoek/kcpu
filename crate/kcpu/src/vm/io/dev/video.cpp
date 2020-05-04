#include "video.hpp"

namespace iodev {

video::video() : rend(graphics::get_graphics().new_renderer(WIDTH * PIXEL_WIDTH, HEIGHT * PIXEL_WIDTH)) {
}

video::~video() {
    delete rend;
}

constexpr unsigned int video::get_framebuffer_size() {
    return WIDTH * PIXEL_WIDTH * HEIGHT * PIXEL_WIDTH * 4;
}

unsigned int video::get_addr() {
    unsigned int addr = (((unsigned int) hiaddr) << 16) | ((unsigned int) loaddr);
    vm_assert(addr < get_framebuffer_size());
    return addr;
}

video::vram_addr video::decode_addr(unsigned int addr) {
    return { .r = addr / (WIDTH * 4), .c = (addr % (WIDTH * 4)) / 4, .comp = addr % 4};
}

std::vector<Word> video::get_reserved_ports() {
    std::vector<Word> ports;
    for(uint i = 0; i < REGISTER_COUNT; i++) {
        ports.push_back(PORT_BASE + i);
    }
    return ports;
}

void video::handle_command(Word cmd) {
    switch(cmd) {
        case CMD_FLIP: {
            rend->get_fb().advance();
            break;
        }
        case CMD_STREAMRST: {
            stream_ptr = 0;
            break;
        }
        default: panic!("unknown video command");
    }
}

void video::vram_write(unsigned int a, Word val) {
    vram_addr addr = decode_addr(a);

    for(uint r = 0; r < PIXEL_WIDTH; r++) {
        for(uint c = 0; c < PIXEL_WIDTH; c++) {
            uint new_r = r + (PIXEL_WIDTH * addr.r);
            uint new_c = c + (PIXEL_WIDTH * addr.c);
            rend->get_fb().get_fb_next()[(new_r * 4 * WIDTH * PIXEL_WIDTH) + (new_c * 4) + addr.comp] = val;
        }
    }
}

Word video::vram_read(unsigned int a) {
    vram_addr addr = decode_addr(a);

    uint new_r = (PIXEL_WIDTH * addr.r);
    uint new_c = (PIXEL_WIDTH * addr.c);
    return rend->get_fb().get_fb_next()[(new_r * 4 * WIDTH * PIXEL_WIDTH) + (new_c * 4) + addr.comp];
}

halfcycle_count_t video::write(Word port, Word val) {
    uint reg = port - PORT_BASE;
    vm_assert(port >= PORT_BASE && reg <= REGISTER_COUNT);

    switch(reg) {
        case REG_CMD: {
            handle_command(val);
            break;
        }
        case REG_STREAM: {
            vram_write(stream_ptr++, val);
            break;
        }
        case REG_HIADDR: {
            hiaddr = val;
            break;
        }
        case REG_LOADDR: {
            loaddr = val;
            break;
        }
        case REG_DATA: {
            vram_write(get_addr(), val);
            break;
        }
        default: panic!("unknown video register");
    }

    return 0;
}

std::pair<Word, halfcycle_count_t> video::read(Word port) {
    uint reg = port - PORT_BASE;
    vm_assert(port >= PORT_BASE && reg <= REGISTER_COUNT);

    switch(reg) {
        case REG_DATA: {
            return std::pair(vram_read(get_addr()), 0);
        }
        case REG_CMD:
        case REG_STREAM:
        case REG_HIADDR:
        case REG_LOADDR: panic!("cannot read from that graphics register");
        default: panic!("unknown graphics register");
    }
}

}