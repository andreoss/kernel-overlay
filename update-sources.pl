#!/usr/bin/env nix-shell
#! nix-shell -i perl -p perl536 -p perl536Packages.JSON -p perl536Packages.FileSlurp -p perl536Packages.Mojolicious -p perl536Packages.DataDump -p perl536Packages.IOSocketSSL -p wget

use strict;
use warnings;
use feature qw(say);
use Carp qw(croak);
use Data::Dump;
use File::Slurp;
use JSON;
use Mojo::DOM;
use Mojo::UserAgent;

my $json = JSON->new->utf8->pretty(1)->canonical(1)->allow_nonref;
my $meta = $json->decode( read_file('meta.json') );
my $ua   = Mojo::UserAgent->new;

$ua->max_redirects(10)->connect_timeout(5)->request_timeout(15);

sub load_all_checksums {
    my $url = shift;
    my $res = $ua->get($url)->result;
    return () unless $res->code == 200;
    my %result = ();
    for my $l ( split /\n/, $res->body ) {
        next unless $l =~ m/([0-9a-f]+)\s+(\S+)/;
        $result{$2} = $1;
    }
    return %result;

}

sub checksum {
    my $all  = shift;
    my $url  = shift;
    my $name = ( split /\//, $url )[-1];
    my %r    = load_all_checksums($all);
    return $r{$name} if exists $r{$name};

    chomp( my ($sha) = `wget -O- $url | sha256sum - | cut -f 1 -d' '` );
    return $sha;
}

my $res = $ua->get('https://kernel.org')->result;

croak unless $res->code == 200;

my $dom = Mojo::DOM->new( $res->body );

my $releases = $dom->at("#releases");

my @sources = ();

sub first_href {
    my ( $obj, $i ) = @_;
    return '' unless $obj || $obj->[$i];
    my $f = $obj->[$i]->find('a[href]')->first;
    return '' unless $f;
    return $f->attr('href');
}

sub trim_version {
    my $version = shift;
    return unless $version =~ m/ ^ (\d+) [.] (\d+) /xgsm;
    return "$1_$2";
}

for my $e ( $releases->find('tr')->each ) {
    my $items    = $e->find('td');
    my $category = $items->[0]->all_text =~ s/\W//gr;
    next unless exists $meta->{categories}{$category};
    my $version   = lc( $items->[1]->all_text =~ s/\s/-/gr =~ s/[\[-\]]//gr );
    my $date      = $items->[2]->all_text;
    my $tarball   = first_href( $items, 3 );
    my $pgp       = first_href( $items, 4 );
    my $browse    = first_href( $items, 8 );
    my $changelog = first_href( $items, 9 );
    my $checksum_url = $tarball =~ s[(?<=/)[^/]+$][sha256sums.asc]rs;
    my $checksum     = checksum( $checksum_url, $tarball );

    if ( $version =~ /rc/i ) {
        $version =~ s/ (\d+) [.] (\d+) [-] (\w+) /$1.$2.0-$3/xgsm;
    }
    my $pversion = trim_version($version);
    if ( $version =~ /eol/i ) {
        $version  =~ s/-eol//i;
        $category = "eol";
    }
    push @sources,
      {
        category => $category,
        checksum => $checksum,
        version  => $version,
        date     => $date,
        url      => $tarball,
        package  => {
            name => ( $category eq 'stable' || $category eq 'mainline' )
            ? $category
            : $pversion
        },
        meta => {
            link      => $browse,
            pgp       => $pgp,
            changelog => $changelog,
        }
      };
}

write_file( 'sources.json',
    $json->encode( [ sort { $a->{version} cmp $b->{version} } @sources ] ) );
